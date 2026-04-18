# Privacy

This document describes the current privacy behavior of Actiona 4 for:

- update checks, which are enabled by default and can be disabled
- telemetry, which is disabled by default and must be enabled explicitly
- crash reports, which are only sent with your explicit consent

## Summary

- Automatic update checks are opt-out.
- Telemetry is opt-in. The code that sends telemetry data has not been implemented yet; enabling it currently only adds a pseudonymous client ID to update-check requests.
- Crash reports are opt-in per crash and do not contain personal information.
- The server only stores a hashed and salted version of the source IP address, not the raw IP address.

## Update Checks

When update checks are enabled, Actiona contacts `https://updates.actiona.app/v1`.

Update checks happen:

- automatically, at most once per day, when automatic update checks are enabled
- when you explicitly run `actiona-run update`
- from the Windows installer if you choose to check for a newer version before installing

The update request can include:

- application name
- application channel
- application version
- operating system name
- operating system distribution on Linux, when available
- operating system version, when available
- CPU architecture
- system locale, when available
- display session type on Linux (`x11`, `xwayland`, or `wayland`) when detectable
- application distribution or package identifier

This data is used to determine whether an update is available and which download should be offered.

## Telemetry

Telemetry is disabled by default.

If you enable telemetry, Actiona generates a random UUID locally and stores it in your local settings as a pseudonymous client ID. That client ID is then included in update-check requests as `client_id`.

The code that sends additional telemetry data has not been implemented yet. No separate telemetry endpoint exists in this version. Enabling telemetry therefore only allows the update service to recognize that multiple update checks came from the same Actiona installation, without directly identifying you by name or email.

## IP Addresses

As with any network request, the service necessarily receives the source IP address at connection time in order to respond.

The server only stores a hashed and salted version of the IP address, not the raw IP address.

## What Is Not Sent By These Requests

Update checks and telemetry requests do not intentionally include:

- your name or email address
- script contents
- file contents
- screenshots
- keyboard input
- mouse input

## Local Settings

Actiona stores its settings locally in a `settings.toml` file managed through the platform-specific application settings directory.

Relevant settings are:

- `update_check`
- `telemetry`

When telemetry is enabled, the `telemetry` setting contains the generated UUID. When telemetry is disabled, that value is removed from the local settings.

Actiona also stores update state locally in a `state.toml` file, including information such as the next scheduled update check and the last update result shown to the user.

## Your Choices

You can change these settings at any time:

```sh
actiona-run config update_check false
actiona-run config update_check true
actiona-run config telemetry false
actiona-run config telemetry true
```

You can also print the current values:

```sh
actiona-run config update_check
actiona-run config telemetry
```

## Crash Reports

Crash reporting is separate from update checks and telemetry.

If Actiona crashes, a dialog will ask whether you want to send a crash report. The report is only sent if you click **Yes**. Crash reports do not contain personal information such as your name, email address, script contents, file contents, screenshots, keyboard input, or mouse input.
