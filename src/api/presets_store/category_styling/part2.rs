use super::CategoryStyling;

pub fn match_styling(name_lower: &str, default_icon: &str) -> Option<CategoryStyling> {
    // Social Media (Pinkish Lilac)
    if name_lower.contains("mastodon") || name_lower.contains("wordpress") ||
       name_lower.contains("ghost") || name_lower.contains("linkding") ||
       name_lower.contains("linkwarden") || name_lower.contains("wallabag") ||
       name_lower.contains("shiori") || name_lower.contains("yourls") ||
       name_lower.contains("kutt") || name_lower.contains("humhub") ||
       name_lower.contains("friendica") {
        return Some(CategoryStyling {
            icon: default_icon.to_string(),
            color: "#ff9ff3",
            bg: "rgba(255, 159, 243, 0.08)",
            border: "rgba(255, 159, 243, 0.2)",
        });
    }

    // Backup (Teal)
    if name_lower.contains("duplicati") || name_lower.contains("duplicacy") || name_lower.contains("kopia") ||
       name_lower.contains("backups") || name_lower.contains("archivebox") || name_lower.contains("restic") ||
       name_lower.contains("borgbackup") || name_lower.contains("urbackup") {
        return Some(CategoryStyling {
            icon: default_icon.to_string(),
            color: "#00d2d3",
            bg: "rgba(0, 210, 211, 0.08)",
            border: "rgba(0, 210, 211, 0.2)",
        });
    }

    // Sync (Mint Green)
    if name_lower.contains("syncthing") || name_lower.contains("rclone") || name_lower.contains("krusader") ||
       name_lower.contains("filezilla") || name_lower.contains("rsync") || name_lower.contains("resilio-sync") {
        return Some(CategoryStyling {
            icon: default_icon.to_string(),
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
        return Some(CategoryStyling {
            icon: default_icon.to_string(),
            color: "#6c7a89",
            bg: "rgba(108, 122, 137, 0.08)",
            border: "rgba(108, 122, 137, 0.2)",
        });
    }

    None
}
