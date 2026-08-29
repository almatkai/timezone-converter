import { DateTime, IANAZone } from "luxon";

export function formatZoneName(zoneName: string): string {
  return zoneName?.replaceAll("/", " - ").replaceAll("_", " ");
}

export function getTimezoneOffsetString(zone?: string): string | undefined {
  if (!zone) return undefined;
  try {
    const offset = IANAZone.create(zone).offset(Date.now()) / 60;
    const isNegative = offset < 0;
    return isNegative ? `GMT${offset}` : `GMT+${offset}`;
  } catch {
    return ``;
  }
}

export function markdownForTime(now: DateTime, isCustom?: boolean): string {
  const offset = now.offset / 60;
  const isNegative = offset < 0;
  const isZero = offset === 0;
  const offsetString = isNegative ? String(offset) : isZero ? `` : `+${offset}`;
  return `
# ${isCustom ? "Custom time" : "Local time"}
## ${now.toFormat("ff")}
${formatZoneName(now.zoneName)} (${now.toFormat("ZZZZ")})
![${formatZoneName(now.zoneName)}](timezones/UTC${offsetString}.png)
`;
}

export function convertToZone(date: DateTime, zone: string): DateTime {
  return date.setZone(zone);
}

export function getAllTimezones(): string[] {
  const maybeSupported = (Intl as unknown as { supportedValuesOf?: (key: string) => string[] }).supportedValuesOf;
  if (typeof maybeSupported === "function") {
    try {
      return ["UTC", ...maybeSupported("timeZone")];
    } catch {
      // fallback
    }
  }
  // Fallback: use Intl timezones if supportedValuesOf unavailable
  return ["UTC"];
}
