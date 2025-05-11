/**
 * Date utilities for TAP messages
 */

/**
 * Get the current Unix timestamp in seconds
 * 
 * @returns Current Unix timestamp in seconds
 */
export function getCurrentUnixTimestamp(): number {
  return Math.floor(Date.now() / 1000);
}

/**
 * Convert a JavaScript Date to an ISO 8601 DateTime string
 * 
 * @param date JavaScript Date object
 * @returns ISO 8601 formatted date string
 */
export function dateToISO8601(date: Date): string {
  return date.toISOString();
}

/**
 * Convert an ISO 8601 DateTime string to a JavaScript Date
 * 
 * @param isoDate ISO 8601 formatted date string
 * @returns JavaScript Date object
 */
export function iso8601ToDate(isoDate: string): Date {
  return new Date(isoDate);
}

/**
 * Create an expiration timestamp that is a specific number of seconds in the future
 * 
 * @param secondsFromNow Number of seconds from now when the timestamp should expire
 * @returns Expiration timestamp in seconds
 */
export function createExpirationTimestamp(secondsFromNow: number): number {
  return getCurrentUnixTimestamp() + secondsFromNow;
}

/**
 * Check if a Unix timestamp has expired
 * 
 * @param timestamp Unix timestamp in seconds
 * @returns True if the timestamp has expired, false otherwise
 */
export function isExpired(timestamp: number): boolean {
  return getCurrentUnixTimestamp() > timestamp;
}