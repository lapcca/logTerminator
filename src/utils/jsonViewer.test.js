import { describe, it, expect } from 'vitest'
import { detectJson } from './jsonViewer.js'

describe('detectJson', () => {
  describe('Python-style single quotes (non-standard JSON)', () => {
    it('should extract JSON object from mixed content with single quotes (test data 1)', () => {
      const rawMessage = `+REST_RESPONSE[THREAD]: 140520979682240 [SEQ]: 30 [STATUS]: 200
[RESPONSE in 0:00:00.133846]:
{'id': '696a16f0-9e30-f08f-884e-c2a4fa5e203c','nas_server': {'id': '696a16d9-f761-f48b-2ef6-c2a4fa5e203c'}, 'ip_port': {'id': 'IP_PORT19'}, 'file_interface_routes': []}`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual({
        id: '696a16f0-9e30-f08f-884e-c2a4fa5e203c',
        nas_server: { id: '696a16d9-f761-f48b-2ef6-c2a4fa5e203c' },
        ip_port: { id: 'IP_PORT19' },
        file_interface_routes: []
      })
    })

    it('should extract JSON array from mixed content with single quotes (test data 2)', () => {
      const rawMessage = `+REST_RESPONSE[THREAD]: 140520979682240 [SEQ]: 30 [STATUS]: 200
[RESPONSE in 0:00:00.133846]:
[{'id': '696a16f0-9e30-f08f-884e-c2a4fa5e203c','nas_server': {'id': '696a16d9-f761-f48b-2ef6-c2a4fa5e203c'}, 'ip_port': {'id': 'IP_PORT19'}, 'file_interface_routes': []}]`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual([
        {
          id: '696a16f0-9e30-f08f-884e-c2a4fa5e203c',
          nas_server: { id: '696a16d9-f761-f48b-2ef6-c2a4fa5e203c' },
          ip_port: { id: 'IP_PORT19' },
          file_interface_routes: []
        }
      ])
    })

    it('should extract JSON with Python literals (True/False/None) (test data 5)', () => {
      const rawMessage = `+REST_RESPONSE[THREAD]: 140520979682240 [SEQ]: 1 [STATUS]: 200
[RESPONSE in 0:00:00.117433]:
[{'is_password_change_required': False, 'user_type': 'Local_User', 'is_built_in_user': True, 'user_id': '1', 'id': '3a07d3ec-513b-4892-a1e4-518af3b89256', 'idle_timeout': 3600, 'user': None, 'role_ids': ['1']}]`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual([
        {
          is_password_change_required: false,
          user_type: 'Local_User',
          is_built_in_user: true,
          user_id: '1',
          id: '3a07d3ec-513b-4892-a1e4-518af3b89256',
          idle_timeout: 3600,
          user: null,
          role_ids: ['1']
        }
      ])
    })
  })

  describe('Standard JSON with double quotes', () => {
    it('should extract JSON object from mixed content with double quotes (test data 3)', () => {
      const rawMessage = `+REST_RESPONSE[THREAD]: 140520979682240 [SEQ]: 30 [STATUS]: 200
[RESPONSE in 0:00:00.133846]:
{"id": "696a16f0-9e30-f08f-884e-c2a4fa5e203c","nas_server": {"id": "696a16d9-f761-f48b-2ef6-c2a4fa5e203c"}, "ip_port": {"id": "IP_PORT19"}, "file_interface_routes": []}`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual({
        id: '696a16f0-9e30-f08f-884e-c2a4fa5e203c',
        nas_server: { id: '696a16d9-f761-f48b-2ef6-c2a4fa5e203c' },
        ip_port: { id: 'IP_PORT19' },
        file_interface_routes: []
      })
    })

    it('should extract JSON array from mixed content with double quotes (test data 4)', () => {
      const rawMessage = `+REST_RESPONSE[THREAD]: 140520979682240 [SEQ]: 30 [STATUS]: 200
[RESPONSE in 0:00:00.133846]:
[{"id": "696a16f0-9e30-f08f-884e-c2a4fa5e203c","nas_server": {"id": "696a16d9-f761-f48b-2ef6-c2a4fa5e203c"}, "ip_port": {"id": "IP_PORT19"}, "file_interface_routes": []}]`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual([
        {
          id: '696a16f0-9e30-f08f-884e-c2a4fa5e203c',
          nas_server: { id: '696a16d9-f761-f48b-2ef6-c2a4fa5e203c' },
          ip_port: { id: 'IP_PORT19' },
          file_interface_routes: []
        }
      ])
    })

    it('should handle pure JSON object with double quotes', () => {
      const rawMessage = `{"id": "test", "value": 123}`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual({ id: 'test', value: 123 })
    })
  })

  describe('Edge cases and error handling', () => {
    it('should handle mixed content with valid JSON (double quotes)', () => {
      const rawMessage = `+REST_RESPONSE[THREAD]: 140520979682240 [SEQ]: 30
{"id": "696a16f0", "nas_server": {"id": "test"}}`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual({
        id: '696a16f0',
        nas_server: { id: 'test' }
      })
    })

    it('should return failure for non-JSON content', () => {
      const rawMessage = `Just a regular log message without any JSON`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(false)
      expect(result.parsed).toBeNull()
      expect(result.error).toBe('No valid JSON found in message')
    })

    it('should handle empty string', () => {
      const result = detectJson('')

      expect(result.success).toBe(false)
      expect(result.parsed).toBeNull()
      expect(result.error).toBe('Invalid message')
    })

    it('should handle null input', () => {
      const result = detectJson(null)

      expect(result.success).toBe(false)
      expect(result.parsed).toBeNull()
      expect(result.error).toBe('Invalid message')
    })

    it('should handle nested objects with single quotes', () => {
      const rawMessage = `{'a': {'b': {'c': 'd'}}, 'e': [1, 2, 3]}`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual({
        a: { b: { c: 'd' } },
        e: [1, 2, 3]
      })
    })

    it('should handle escaped quotes in single-quoted strings', () => {
      const rawMessage = `{'message': 'It\\'s a test', 'value': 123}`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual({
        message: "It's a test",
        value: 123
      })
    })
  })
})
