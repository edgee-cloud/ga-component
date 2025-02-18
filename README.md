<div align="center">
<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/component-dark.svg">
      <img src="https://cdn.edgee.cloud/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>

<h1 align="center">Google Analytics Component for Edgee</h1>

[![Coverage Status](https://coveralls.io/repos/github/edgee-cloud/ga-component/badge.svg)](https://coveralls.io/github/edgee-cloud/ga-component)
[![GitHub issues](https://img.shields.io/github/issues/edgee-cloud/ga-component.svg)](https://github.com/edgee-cloud/ga-component/issues)
[![Edgee Component Registry](https://img.shields.io/badge/Edgee_Component_Registry-Public-green.svg)](https://www.edgee.cloud/edgee/google-analytics)

This component enables seamless integration between [Edgee](https://www.edgee.cloud) and [Google Analytics](https://marketingplatform.google.com/about/analytics/), allowing you to collect and forward analytics data while respecting user privacy settings.

## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `ga.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[components.data_collection]]
id = "google_analytics"
file = "/var/edgee/components/ga.wasm"
settings.ga_measurement_id = "G-XXXXXXXXXX"  # Your GA4 Measurement ID
```

## Event Handling

### Event Mapping
The component maps Edgee events to Google Analytics events as follows:

| Edgee Event | GA4 Event    | Description |
|-------------|--------------|-------------|
| Page        | `page_view`  | Triggered when a user views a page |
| Track       | Custom Event | Uses the provided event name directly |
| User        | N/A         | Used for user identification only |

### User Event Handling
While User events don't generate GA events directly, they serve an important purpose:
- Stores `user_id`, `anonymous_id`, and `properties` on the user's device
- Enriches subsequent Page and Track events with user data
- Enables proper user attribution across sessions

## Configuration Options

### Basic Configuration
```toml
[[components.data_collection]]
id = "google_analytics"
file = "/var/edgee/components/ga.wasm"
settings.ga_measurement_id = "G-XXXXXXXXXX"

# Optional configurations
settings.edgee_anonymization = true        # Enable/disable data anonymization in case of pending or denied consent
settings.edgee_default_consent = "pending" # Set default consent status if not specified by the user
```

### Event Controls
Control which events are forwarded to Google Analytics:
```toml
settings.edgee_page_event_enabled = true   # Enable/disable page event
settings.edgee_track_event_enabled = true  # Enable/disable track event
settings.edgee_user_event_enabled = true   # Enable/disable user event
```

### Consent Management
Before sending events to Google Analytics, you can set the user consent using the Edgee SDK: 
```javascript
edgee.consent("granted");
```

Or using the Data Layer:
```html
<script id="__EDGEE_DATA_LAYER__" type="application/json">
  {
    "data_collection": {
      "consent": "granted"
    }
  }
</script>
```

If the consent is not set, the component will use the default consent status.

| Consent | Anonymization | Google Analytics Consent |
|---------|---------------|--------------------------|
| pending | true          | analytics only           |
| denied  | true          | analytics only           |
| granted | false         | fully granted            |

## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)
- WASM target: `rustup target add wasm32-wasip2`
- wit-deps: `cargo install wit-deps`
- cargo-llvm-cov: `cargo install cargo-llvm-cov`

Build command:
```bash
make wit-deps
make build
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)
