/**
 * Stack trace parser utilities for Python stack traces
 */

/**
 * Regular expression for parsing Python stack frames
 * Matches: File "path/to/file.py", line 123, in function_name
 *         source code line
 */
const STACK_FRAME_REGEX = /File "([^"]+)", line (\d+), in (\S+)\s*\n\s+(.+)/g;

/**
 * Parsed stack frame interface
 * @typedef {Object} ParsedStackFrame
 * @property {string} file - Full file path
 * @property {string} line - Line number as string
 * @property {string} function - Function name
 * @property {string} code - Source code line
 */

/**
 * Parse Python stack trace into structured array
 *
 * @param {string} stackRaw - Raw stack trace string
 * @returns {ParsedStackFrame[]} Array of parsed stack frames, empty if parsing fails
 */
export function parsePythonStackTrace(stackRaw) {
  if (!stackRaw || typeof stackRaw !== 'string') {
    return [];
  }

  const frames = [];
  let match;

  // Reset regex state
  STACK_FRAME_REGEX.lastIndex = 0;

  while ((match = STACK_FRAME_REGEX.exec(stackRaw)) !== null) {
    frames.push({
      file: match[1],
      line: match[2],
      function: match[3],
      code: match[4]?.trim() || ''
    });
  }

  return frames;
}

/**
 * Get a preview of the stack trace (first line or first few frames)
 *
 * @param {string} stackRaw - Raw stack trace string
 * @returns {string} Preview text
 */
export function getStackPreview(stackRaw) {
  if (!stackRaw || typeof stackRaw !== 'string') {
    return '-';
  }

  const lines = stackRaw.trim().split('\n');
  const firstLine = lines[0]?.trim();

  if (!firstLine) {
    return '-';
  }

  // If first line looks like a Python file reference, show it
  if (firstLine.includes('File "') || firstLine.includes('line ')) {
    const frames = parsePythonStackTrace(stackRaw);
    if (frames.length > 0) {
      return `${frames.length} frames - 点击查看调用栈`;
    }
  }

  // For short stack traces, show first line truncated
  if (firstLine.length > 50) {
    return firstLine.substring(0, 50) + '...';
  }

  return firstLine || '-';
}

/**
 * Check if a stack trace appears to be a Python stack trace
 *
 * @param {string} stackRaw - Raw stack trace string
 * @returns {boolean} True if appears to be Python stack trace
 */
export function isPythonStackTrace(stackRaw) {
  if (!stackRaw || typeof stackRaw !== 'string') {
    return false;
  }

  // Check for Python stack trace indicators
  return stackRaw.includes('File "') &&
         (stackRaw.includes('line ') || stackRaw.includes(', in '));
}
