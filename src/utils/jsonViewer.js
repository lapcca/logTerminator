/**
 * Extract JSON content from text using bracket counting
 * Handles nested structures, strings, escape sequences, and line breaks
 * @param {string} text - The text to extract JSON from
 * @param {string} startChar - The opening bracket '{' or '['
 * @param {string} endChar - The closing bracket '}' or ']'
 * @returns {string|null} Extracted JSON string or null
 */
function extractJsonByBrackets(text, startChar, endChar) {
  for (let i = 0; i < text.length; i++) {
    if (text[i] === startChar) {
      let depth = 0
      let inString = false
      let escapeNext = false
      let start = i

      for (let j = i; j < text.length; j++) {
        const char = text[j]

        // Handle escape sequences inside strings
        if (escapeNext) {
          escapeNext = false
          continue
        }

        // Track escape characters
        if (char === '\\' && inString) {
          escapeNext = true
          continue
        }

        // Track string boundaries
        if (char === '"') {
          inString = !inString
          continue
        }

        // Only count brackets when not inside a string
        if (!inString) {
          if (char === startChar) {
            depth++
          } else if (char === endChar) {
            depth--
            if (depth === 0) {
              // Found complete JSON structure
              return text.substring(start, j + 1)
            }
          }
        }
      }
      // If we exit the loop without finding the closing bracket,
      // this wasn't a valid JSON start, continue searching
    }
  }
  return null
}

/**
 * Extract all potential JSON candidates from text
 * Prioritizes arrays containing objects (e.g., [{...}]), then objects, then bare arrays
 * @param {string} text - The text to search
 * @returns {Array<string>} Array of potential JSON strings
 */
function extractJsonCandidates(text) {
  const candidates = []

  const arrayCandidate = extractCompleteStructure(text, '[', ']')
  const objectCandidate = extractCompleteStructure(text, '{', '}')

  // If array starts with '[{' (array of objects), prioritize it over objects
  if (arrayCandidate && arrayCandidate.trim().startsWith('[{')) {
    candidates.push(arrayCandidate)
  }

  // Then add objects
  if (objectCandidate) {
    candidates.push(objectCandidate)
  }

  // Finally add arrays (if not already added and different from object)
  if (arrayCandidate && arrayCandidate !== objectCandidate) {
    // Only add if it's a pure array (not starting with [{ which we already added)
    if (!arrayCandidate.trim().startsWith('[{')) {
      candidates.push(arrayCandidate)
    }
  }

  return candidates
}

/**
 * Extract a complete JSON structure from text using bracket counting
 * This version returns the largest complete structure found
 * @param {string} text - The text to search
 * @param {string} startChar - Opening bracket ([ or {)
 * @param {string} endChar - Closing bracket (] or })
 * @returns {string|null} Extracted JSON string or null
 */
function extractCompleteStructure(text, startChar, endChar) {
  let bestMatch = null
  let bestMatchLength = 0

  for (let i = 0; i < text.length; i++) {
    if (text[i] === startChar) {
      let depth = 0
      let inString = false
      let escapeNext = false
      let start = i
      let end = -1

      for (let j = i; j < text.length; j++) {
        const char = text[j]

        // Handle escape sequences inside strings
        if (escapeNext) {
          escapeNext = false
          continue
        }

        // Track escape characters
        if (char === '\\' && inString) {
          escapeNext = true
          continue
        }

        // Track string boundaries
        if (char === '"') {
          inString = !inString
          continue
        }

        // Only count brackets when not inside a string
        if (!inString) {
          if (char === startChar) {
            depth++
          } else if (char === endChar) {
            depth--
            if (depth === 0) {
              end = j
              break
            }
          }
        }
      }

      // If we found a complete structure
      if (end !== -1) {
        const candidate = text.substring(start, end + 1)
        // Keep the longest match (to prefer larger structures)
        if (candidate.length > bestMatchLength) {
          bestMatch = candidate
          bestMatchLength = candidate.length
        }
      }
    }
  }

  return bestMatch
}

/**
 * Convert Python-style single-quoted dict/array to valid JSON
 * Handles escaped quotes and preserves string content
 * Also converts Python literals (True, False, None) to JSON (true, false, null)
 * @param {string} str - Python-style string with single quotes
 * @returns {string} Valid JSON string with double quotes
 */
function convertSingleQuotesToJson(str) {
  let result = ''
  let inString = false
  let escapeNext = false

  for (let i = 0; i < str.length; i++) {
    const char = str[i]

    if (escapeNext) {
      // Handle escaped characters
      if (char === "'") {
        // Python's \' becomes just ' in JSON (single quotes don't need escaping in double-quoted strings)
        result += "'"
      } else if (char === '\\') {
        // \\ becomes \\
        result += '\\\\'
      } else if (char === 'n') {
        // \n becomes \n
        result += '\\n'
      } else if (char === 'r') {
        // \r becomes \r
        result += '\\r'
      } else if (char === 't') {
        // \t becomes \t
        result += '\\t'
      } else {
        // Other escaped characters
        result += '\\' + char
      }
      escapeNext = false
      continue
    }

    if (char === '\\' && inString) {
      escapeNext = true
      continue
    }

    if (char === "'") {
      // Replace single quote with double quote (string delimiter)
      result += '"'
      inString = !inString
    } else if (!inString) {
      // Convert Python literals to JSON literals (only when not in string)
      // Check for True/False/None as complete words
      if (char === 'T' && str.substr(i, 4) === 'True') {
        result += 'true'
        i += 3
      } else if (char === 'F' && str.substr(i, 5) === 'False') {
        result += 'false'
        i += 4
      } else if (char === 'N' && str.substr(i, 4) === 'None') {
        result += 'null'
        i += 3
      } else {
        result += char
      }
    } else {
      result += char
    }
  }

  return result
}

/**
 * Detect if a message contains valid JSON
 * First attempts direct parse, then bracket counting extraction for mixed content
 * Handles both JSON (double quotes) and Python-style (single quotes) formats
 * @param {string} message - The message to check
 * @returns {Object} { success: boolean, parsed: object|null, error: string|null }
 */
export function detectJson(message) {
  if (!message || typeof message !== 'string') {
    return { success: false, parsed: null, error: 'Invalid message' }
  }

  const trimmed = message.trim()

  // First try: direct parse (for pure JSON messages)
  try {
    const parsed = JSON.parse(trimmed)
    return {
      success: true,
      parsed: parsed,
      error: null
    }
  } catch (e) {
    // Continue to extraction approach
  }

  // Second try: extract JSON from mixed content using bracket counting
  const candidates = extractJsonCandidates(message)

  for (const candidate of candidates) {
    // Try standard JSON first
    try {
      const parsed = JSON.parse(candidate)
      return {
        success: true,
        parsed: parsed,
        error: null
      }
    } catch (e) {
      // Try converting single quotes to double quotes (Python-style)
      try {
        const converted = convertSingleQuotesToJson(candidate)
        const parsed = JSON.parse(converted)
        return {
          success: true,
          parsed: parsed,
          error: null
        }
      } catch (e2) {
        // Try next candidate
      }
    }
  }

  // No valid JSON found
  return {
    success: false,
    parsed: null,
    error: 'No valid JSON found in message'
  }
}

/**
 * Syntax highlight JSON with HTML spans
 * @param {*} jsonObj - The parsed JSON object
 * @param {number} depth - Current nesting depth
 * @returns {string} HTML string with syntax highlighting
 */
export function syntaxHighlightJson(jsonObj, depth = 0) {
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

    let output = '<div class="json-array" data-depth="' + depth + '">'
    output += '<span class="json-toggle" onclick="toggleJsonNode(this)">[-]</span> ['

    for (let i = 0; i < jsonObj.length; i++) {
      output += '<div class="json-item" data-collapsed="false">'
      output += syntaxHighlightJson(jsonObj[i], depth + 1)
      if (i < jsonObj.length - 1) {
        output += ','
      }
      output += '</div>'
    }

    output += ']</div>'
    return output
  }

  if (typeof jsonObj === 'object') {
    const keys = Object.keys(jsonObj)
    if (keys.length === 0) {
      return '{}'
    }

    let output = '<div class="json-object" data-depth="' + depth + '">'
    output += '<span class="json-toggle" onclick="toggleJsonNode(this)">[-]</span> {'

    for (let i = 0; i < keys.length; i++) {
      const key = keys[i]
      output += '<div class="json-item" data-collapsed="false">'
      output += '<span style="color: #9cdcfe">"' + escapeHtml(key) + '"</span>'
      output += ': '
      output += syntaxHighlightJson(jsonObj[key], depth + 1)
      if (i < keys.length - 1) {
        output += ','
      }
      output += '</div>'
    }

    output += '}</div>'
    return output
  }

  return String(jsonObj)
}

/**
 * Toggle JSON node collapse/expand
 * @param {HTMLElement} element - The toggle button element
 */
// Only assign to window in browser environment
if (typeof window !== 'undefined') {
  window.toggleJsonNode = function(element) {
  const parent = element.parentElement
  const items = parent.querySelectorAll(':scope > .json-item')
  const isCollapsed = parent.getAttribute('data-collapsed') === 'true'

  if (isCollapsed) {
    // Expand
    items.forEach(item => item.style.display = '')
    parent.setAttribute('data-collapsed', 'false')
    element.textContent = '[-]'
  } else {
    // Collapse
    items.forEach(item => item.style.display = 'none')
    parent.setAttribute('data-collapsed', 'true')
    element.textContent = '[+]'
  }
}
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
