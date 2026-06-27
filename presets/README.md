# Nix Service Presets

This directory contains pre-defined metadata configurations for common Nix packages. 
Storing presets as separate JSON files allows the community to easily contribute new service definitions by simply adding a JSON file here.

## Schema Format

Each preset file should be named `<package-name>.json` (lowercase) and follow this structure:

```json
{
  "name": "radarr",
  "display_name": "Radarr",
  "description": "Radarr Movie Manager",
  "url": "https://radarr.video/",
  "default_ports": [
    {
      "host": 7878,
      "container": 7878,
      "label": "HTTP"
    }
  ]
}
```

### Fields:
* `name`: The exact lowercase name of the package as it appears on Nix (e.g. `jellyfin` or `syncthing`).
* `display_name`: Human-readable name of the service.
* `description`: A short description explaining what the service does.
* `url`: A link to the official project homepage.
* `default_ports`: An array of default network ports used by the application:
  * `host`: The default port on the host machine.
  * `container`: The default port inside the container/chroot sandbox.
  * `label`: A label describing the port's protocol/purpose (e.g. `HTTP`, `Sync Protocol`, `DLNA (UDP)`).
