use super::StaticConfig;

pub fn match_static_config(name_lower: &str) -> Option<StaticConfig> {
    // Cloud (Blue)
    if name_lower.contains("nextcloud") || name_lower.contains("owncloud") ||
       name_lower.contains("seafile") || name_lower.contains("filerun") || name_lower.contains("immich") ||
       name_lower.contains("photoprism") || name_lower.contains("komga") || name_lower.contains("uboquity") {
        return Some(StaticConfig {
            icon: "fa-cloud",
            color: "#74b9ff",
            bg: "rgba(116, 185, 255, 0.08)",
            border: "rgba(116, 185, 255, 0.2)",
        });
    }

    // Communication & Chat (Teal)
    if name_lower.contains("matrix-synapse") || name_lower.contains("mattermost") ||
       name_lower.contains("rocketchat") || name_lower.contains("mumble") ||
       name_lower.contains("teamspeak") || name_lower.contains("discourse") ||
       name_lower.contains("mailserver") || name_lower.contains("postfix") {
        return Some(StaticConfig {
            icon: "fa-comments-o",
            color: "#00d2d3",
            bg: "rgba(0, 210, 211, 0.08)",
            border: "rgba(0, 210, 211, 0.2)",
        });
    }

    // Social Media (Pinkish Lilac)
    if name_lower.contains("mastodon") || name_lower.contains("wordpress") ||
       name_lower.contains("ghost") || name_lower.contains("linkding") ||
       name_lower.contains("linkwarden") || name_lower.contains("wallabag") ||
       name_lower.contains("shiori") || name_lower.contains("yourls") ||
       name_lower.contains("kutt") || name_lower.contains("humhub") ||
       name_lower.contains("friendica") {
        return Some(StaticConfig {
            icon: "fa-share-alt",
            color: "#ff9ff3",
            bg: "rgba(255, 159, 243, 0.08)",
            border: "rgba(255, 159, 243, 0.2)",
        });
    }

    // Backup (Teal)
    if name_lower.contains("duplicati") || name_lower.contains("duplicacy") || name_lower.contains("kopia") ||
       name_lower.contains("backups") || name_lower.contains("archivebox") || name_lower.contains("restic") ||
       name_lower.contains("borgbackup") || name_lower.contains("urbackup") {
        return Some(StaticConfig {
            icon: "fa-database",
            color: "#00d2d3",
            bg: "rgba(0, 210, 211, 0.08)",
            border: "rgba(0, 210, 211, 0.2)",
        });
    }

    // Sync (Mint Green)
    if name_lower.contains("syncthing") || name_lower.contains("rclone") || name_lower.contains("krusader") ||
       name_lower.contains("filezilla") || name_lower.contains("rsync") || name_lower.contains("resilio-sync") {
        return Some(StaticConfig {
            icon: "fa-refresh",
            color: "#1abc9c",
            bg: "rgba(26, 188, 156, 0.08)",
            border: "rgba(26, 188, 156, 0.2)",
        });
    }

    // Databases & Monitoring (Grey-Blue)
    if name_lower.contains("influx") || name_lower.contains("prometheus") || name_lower.contains("grafana") ||
       name_lower.contains("kuma") || name_lower.contains("netdata") || name_lower.contains("postgres") ||
       name_lower.contains("mysql") || name_lower.contains("mariadb") || name_lower.contains("redis") ||
       name_lower.contains("mongo") {
        return Some(StaticConfig {
            icon: "fa-bar-chart",
            color: "#6c7a89",
            bg: "rgba(108, 122, 137, 0.08)",
            border: "rgba(108, 122, 137, 0.2)",
        });
    }

    None
}
