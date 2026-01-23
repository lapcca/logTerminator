/**
 * Detect if a message contains valid JSON
 * @param {string} message - The message to check
 * @returns {Object} { success: boolean, parsed: object|null, error: string|null }
 */
export function detectJson(message) {
  if (!message || typeof message !== 'string') {
    return { success: false, parsed: null, error: 'Invalid message' }
  }

  const trimmed = message.trim()

  try {
    const parsed = JSON.parse(trimmed)
    return {
      success: true,
      parsed: parsed,
      error: null
    }
  } catch (e) {
    return {
      success: false,
      parsed: null,
      error: e.message
    }
  }
}

/**
 * Syntax highlight JSON with HTML spans
 * @param {*} jsonObj - The parsed JSON object
 * @returns {string} HTML string with syntax highlighting
 */
export function syntaxHighlightJson(jsonObj) {
  if (jsonObj === null) {
    return '<span style="color: #569cd6">null</span>'
  }

  if (typeof jsonObj === 'boolean') {
    return '<span style="color: #569cd6">' + jsonObj + '</span>'
  }

  if (typeof jsonObj === 'number') {
    return '<span style="color: #b5cea8">' + jsonObj + '</span>'
  }

  if (typeof jsonObj === 'string') {
    return '<span style="color: #ce9178">"' + escapeHtml(jsonObj) + '"</span>'
  }

  if (Array.isArray(jsonObj)) {
    if (jsonObj.length === 0) {
      return '[]'
    }

    let output = '['
    for (let i = 0; i < jsonObj.length; i++) {
      output += '<div>'
      output += syntaxHighlightJson(jsonObj[i])
      if (i < jsonObj.length - 1) {
        output += ','
      }
      output += '</div>'
    }
    output += ']'
    return output
  }

  if (typeof jsonObj === 'object') {
    const keys = Object.keys(jsonObj)
    if (keys.length === 0) {
      return '{}'
    }

    let output = '{'
    for (let i = 0; i < keys.length; i++) {
      const key = keys[i]
      output += '<div>'
      output += '<span style="color: #9cdcfe">"' + escapeHtml(key) + '"</span>'
      output += ': '
      output += syntaxHighlightJson(jsonObj[key])
      if (i < keys.length - 1) {
        output += ','
      }
      output += '</div>'
    }
    output += '}'
    return output
  }

  return String(jsonObj)
}

/**
 * Escape HTML special characters
 * @param {string} str - String to escape
 * @returns {string} Escaped string
 */
function escapeHtml(str) {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

/**
 * Prettify JSON for display
 * @param {string} jsonString - Valid JSON string
 * @returns {string} Prettified JSON string
 */
export function prettifyJson(jsonString) {
  try {
    const parsed = JSON.parse(jsonString)
    return JSON.stringify(parsed, null, 2)
  } catch (e) {
    return jsonString
  }
}

/**
 * Get JSON size in KB
 * @param {string} jsonString - JSON string
 * @returns {number} Size in KB
 */
export function getJsonSize(jsonString) {
  return new Blob([jsonString]).size / 1024
}

/**
 * Search in JSON object
 * @param {*} jsonObj - Parsed JSON object
 * @param {string} searchTerm - Term to search for
 * @returns {Array<string>} Paths to matching properties
 */
export function searchInJson(jsonObj, searchTerm) {
  const results = []
  const lowerSearchTerm = searchTerm.toLowerCase()

  function search(obj, path = '') {
    if (obj === null || typeof obj !== 'object') {
      const valueStr = String(obj).toLowerCase()
      if (valueStr.includes(lowerSearchTerm)) {
        results.push(path || 'value')
      }
      return
    }

    if (Array.isArray(obj)) {
      obj.forEach((item, index) => {
        search(item, path ? `${path}[${index}]` : `[${index}]`)
      })
    } else {
      Object.keys(obj).forEach(key => {
        const keyLower = key.toLowerCase()
        if (keyLower.includes(lowerSearchTerm)) {
          results.push(path ? `${path}.${key}` : key)
        }
        search(obj[key], path ? `${path}.${key}` : key)
      })
    }
  }

  try {
    const parsed = typeof jsonObj === 'object' ? jsonObj : JSON.parse(jsonObj)
    search(parsed)
  } catch (e) {
    // Return empty on parse error
  }

  return results
}
