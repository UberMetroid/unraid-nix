$(function() {
    window.nixServiceHistory = window.nixServiceHistory || {};
    window.nixServiceLastIO = window.nixServiceLastIO || {};
    window.nixServiceLastTime = window.nixServiceLastTime || 0;
    
    function formatBytesRate(bytesPerSec) {
        if (bytesPerSec < 1024) return bytesPerSec.toFixed(1) + ' B/s';
        var kb = bytesPerSec / 1024;
        if (kb < 1024) return kb.toFixed(1) + ' KB/s';
        var mb = kb / 1024;
        if (mb < 1024) return mb.toFixed(1) + ' MB/s';
        var gb = mb / 1024;
        return gb.toFixed(1) + ' GB/s';
    }
    
    function drawSparkline(svg, history, minVal, maxVal, color) {
        if (!svg) return;
        if (history.length < 2) {
            svg.innerHTML = '';
            return;
        }
        var width = 60;
        var height = 12;
        var points = [];
        var range = maxVal - minVal;
        
        for (var i = 0; i < history.length; i++) {
            var x = (i / (history.length - 1)) * width;
            var val = history[i];
            var norm = range > 0 ? (val - minVal) / range : 0;
            norm = Math.max(0, Math.min(1, norm));
            var y = height - (norm * (height - 2)) - 1;
            points.push(x.toFixed(1) + "," + y.toFixed(1));
        }
        var pathData = "M" + points.join(" L");
        
        var path = svg.querySelector('path');
        if (!path) {
            path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            svg.appendChild(path);
        }
        path.setAttribute('d', pathData);
        path.setAttribute('fill', 'none');
        path.setAttribute('stroke', color);
        path.setAttribute('stroke-width', '1.5');
        path.setAttribute('stroke-linecap', 'round');
        path.setAttribute('stroke-linejoin', 'round');
    }

    function pollStats() {
        var now = Date.now();
        var timeDiffSec = window.nixServiceLastTime ? (now - window.nixServiceLastTime) / 1000.0 : 2.0;
        window.nixServiceLastTime = now;

        $.getJSON('/plugins/nix/api.php?action=get_dashboard_json', function(data) {
            if (!Array.isArray(data)) return;
            
            data.forEach(function(svc) {
                var name = svc.name;
                var cpuVal = svc.cpu || 0.0;
                var memVal = svc.mem || 0;
                var memMb = memVal / 1048576.0;
                var currentRead = svc.io_read || 0;
                var currentWrite = svc.io_write || 0;
                
                if (!window.nixServiceHistory[name]) {
                    window.nixServiceHistory[name] = { cpu: [], ram: [], ioIn: [], ioOut: [] };
                }
                if (!window.nixServiceLastIO[name]) {
                    window.nixServiceLastIO[name] = { read: currentRead, write: currentWrite };
                }
                
                var prevIO = window.nixServiceLastIO[name];
                var readDiff = currentRead >= prevIO.read ? currentRead - prevIO.read : 0;
                var writeDiff = currentWrite >= prevIO.write ? currentWrite - prevIO.write : 0;
                
                var readRate = timeDiffSec > 0 ? readDiff / timeDiffSec : 0;
                var writeRate = timeDiffSec > 0 ? writeDiff / timeDiffSec : 0;
                
                window.nixServiceLastIO[name] = { read: currentRead, write: currentWrite };
                
                var svcHist = window.nixServiceHistory[name];
                svcHist.cpu.push(cpuVal);
                svcHist.ram.push(memMb);
                svcHist.ioIn.push(readRate);
                svcHist.ioOut.push(writeRate);
                
                if (svcHist.cpu.length > 20) svcHist.cpu.shift();
                if (svcHist.ram.length > 20) svcHist.ram.shift();
                if (svcHist.ioIn.length > 20) svcHist.ioIn.shift();
                if (svcHist.ioOut.length > 20) svcHist.ioOut.shift();
                
                var cpuRow = $('.nix-stat-row[data-service="' + name + '"][data-type="cpu"]');
                if (cpuRow.length) {
                    cpuRow.find('.nix-stat-val').text(cpuVal.toFixed(1) + '%');
                    var cpuSvg = cpuRow.find('.nix-sparkline')[0];
                    if (cpuSvg) {
                        var maxCpu = Math.max.apply(null, svcHist.cpu);
                        maxCpu = Math.max(10, maxCpu);
                        drawSparkline(cpuSvg, svcHist.cpu, 0, maxCpu, '#00d5ff');
                    }
                }
                
                var ramRow = $('.nix-stat-row[data-service="' + name + '"][data-type="ram"]');
                if (ramRow.length) {
                    ramRow.find('.nix-stat-val').text(memMb.toFixed(1) + ' MB');
                    var ramSvg = ramRow.find('.nix-sparkline')[0];
                    if (ramSvg) {
                        var minRam = Math.min.apply(null, svcHist.ram);
                        var maxRam = Math.max.apply(null, svcHist.ram);
                        var diff = maxRam - minRam;
                        if (diff < 1.0) {
                            minRam = Math.max(0, minRam - 5);
                            maxRam = maxRam + 5;
                        } else {
                            minRam = Math.max(0, minRam - diff * 0.1);
                            maxRam = maxRam + diff * 0.1;
                        }
                        drawSparkline(ramSvg, svcHist.ram, minRam, maxRam, '#d946ef');
                    }
                }
                
                var ioInRow = $('.nix-stat-row[data-service="' + name + '"][data-type="io-in"]');
                if (ioInRow.length) {
                    ioInRow.find('.nix-stat-val').text(formatBytesRate(readRate));
                    var ioInSvg = ioInRow.find('.nix-sparkline')[0];
                    if (ioInSvg) {
                        var maxIoIn = Math.max.apply(null, svcHist.ioIn);
                        maxIoIn = Math.max(1024, maxIoIn);
                        drawSparkline(ioInSvg, svcHist.ioIn, 0, maxIoIn, '#2ecc71');
                    }
                }
                
                var ioOutRow = $('.nix-stat-row[data-service="' + name + '"][data-type="io-out"]');
                if (ioOutRow.length) {
                    ioOutRow.find('.nix-stat-val').text(formatBytesRate(writeRate));
                    var ioOutSvg = ioOutRow.find('.nix-sparkline')[0];
                    if (ioOutSvg) {
                        var maxIoOut = Math.max.apply(null, svcHist.ioOut);
                        maxIoOut = Math.max(1024, maxIoOut);
                        drawSparkline(ioOutSvg, svcHist.ioOut, 0, maxIoOut, '#e67e22');
                    }
                }

                var gpuRow = $('.nix-stat-row[data-service="' + name + '"][data-type="gpu"]');
                if (gpuRow.length && svc.gpu_stats) {
                    var stats = Object.values(svc.gpu_stats);
                    if (stats.length > 0) {
                        gpuRow.find('.nix-stat-val').text(stats[0].sm + '%');
                    }
                }

                var gpuMemRow = $('.nix-stat-row[data-service="' + name + '"][data-type="gpu-mem"]');
                if (gpuMemRow.length && svc.gpu_stats) {
                    var stats = Object.values(svc.gpu_stats);
                    if (stats.length > 0) {
                        gpuMemRow.find('.nix-stat-val').text(stats[0].mem + '%');
                    }
                }
                
                var card = $('.nix-service-card[data-name="' + name + '"]');
                if (card.length) {
                    var isRunning = svc.status.toLowerCase() === 'running';
                    var dot = card.find('.status-dot');
                    if (dot.length) {
                        dot.attr('class', 'status-dot ' + (isRunning ? 'status-running' : 'status-stopped'));
                        dot.attr('title', isRunning ? 'RUNNING' : 'STOPPED');
                    }
                }
            });
        });
    }
    
    if (window.nixServicesInterval) clearInterval(window.nixServicesInterval);
    pollStats();
    window.nixServicesInterval = setInterval(pollStats, 2000);
});
