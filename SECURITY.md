# Security and hardware-safety reports

Include the affected version and a concise impact description. Avoid posting exploit packets, secrets, raw backups or personal data publicly.

Use GitHub private vulnerability reporting if enabled. Otherwise, open an issue titled **Security contact requested**, without sensitive details, so the maintainer can arrange a private channel. Never attach the proprietary Windows software.

Current development focuses on the latest 0.4 release. Do not use 0.1–0.3 for mapping/per-key writes: the ANSI bottom-row assignment was corrected. Revision coverage and persistence remain limited; see the feature matrix.

Startup, hotplug, local save/load and ordinary tests must never configure hardware. Please report regressions in that boundary, permissions, packet bounds, protected keys, backups/restoration or readback/error handling.
