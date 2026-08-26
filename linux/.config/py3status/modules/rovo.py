import json
import os
import subprocess
import urllib.request


class Py3status:
    cache_timeout = 5
    color_good = "#989898"
    color_bad = "#be2422"

    API_URL = (
        "https://listenagile.atlassian.net/"
        "gateway/api/rovodev/v3/credits/entitlements/entitlement-allowance"
    )

    PAYLOAD = json.dumps({
        "cloudId": "23c2f087-ec25-4876-a665-6d8b12f56dba",
        "entitlementId": "453d48ff-1ecd-3603-abab-89b205647850",
        "productKey": "unknown",
    }).encode()

    def rovo(self):
        token = os.environ.get("ATLASSIAN_COOKIE")

        if not token:
            return {
                "full_text": "Rovo: --",
                "color": self.color_bad,
            }

        request = urllib.request.Request(
            self.API_URL,
            data=self.PAYLOAD,
            headers={
                "Accept": "*/*",
                "Content-Type": "application/json",
                "Cookie": f"tenant.session.token={token}",
            },
            method="POST",
        )

        try:
            with urllib.request.urlopen(request, timeout=3) as response:
                data = json.load(response)

            usage = data["currentUsage"]
            cap = data["creditCap"]
            percentage = usage / cap * 100

            return {
                "full_text": f"Rovo: {usage}/{cap} ({percentage:.2f}%)",
                "color": self.color_good,
            }

        except Exception:
            return {
                "full_text": "Rovo: --",
                "color": self.color_bad,
            }

    def on_click(self, i3s_output_list, i3s_config, event):
        if event["button"] == 1:
            subprocess.Popen([
                "xdg-open",
                "https://listenagile.atlassian.net/rovodev/your-usage",
            ])
