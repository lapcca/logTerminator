import { ref } from 'vue'

/**
 * Reserved for future keyword highlighting rules.
 * Each rule should have: { pattern: string|RegExp, color: string, bold: boolean }
 */
const highlightRules = ref([])

/**
 * Format message for display.
 * Currently returns the message as-is (CSS handles newlines via white-space: pre-wrap).
 * Future enhancement: apply highlightRules to wrap keywords with colored spans.
 *
 * @param {string} message - The raw message text
 * @returns {string} The formatted message
 */
export function formatMessage(message) {
  if (!message) return ''

  // Future: Apply highlight rules to add color/bold styling
  // Example:
  // let formatted = message
  // for (const rule of highlightRules.value) {
  //   formatted = formatted.replace(rule.pattern, `<span style="color: ${rule.color};">${rule.pattern}</span>`)
  // }
  // return formatted

  return message
}

/**
 * Set keyword highlighting rules for future use.
 * Rules will be applied in formatMessage() when implemented.
 *
 * @param {Array} rules - Array of highlighting rules
 */
export function setHighlightRules(rules) {
  highlightRules.value = rules
}

/**
 * Get current highlight rules.
 *
 * @returns {Array} Current highlight rules
 */
export function getHighlightRules() {
  return highlightRules.value
}
