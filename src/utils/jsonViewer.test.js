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

    it('should extract JSON from complex REST_RESPONSE message with nested structures', () => {
      const rawMessage = `+REST_RESPONSE[THREAD]: 139715504723776 [SEQ]: 276 [STATUS]: 200
[RESPONSE in 0:00:00.157856]:
{"id": "69872124-5668-59d5-2e21-eebf62ba21c6", "state": "OK", "role": "Destination", "resource_type": "nas_server", "data_transfer_state": "Synchronous", "type": "Synchronous", "last_sync_timestamp": null, "local_resource_id": "69872124-e0ba-c439-acf8-eebf62ba21c6", "remote_resource_id": "6987203e-3aac-ca7a-477d-eebf62ba21c6", "remote_system_id": "395c39c8-2d92-43b7-a08f-b9459f785b39", "failover_snapshot_id": null, "failover_snapshot_expiration_timestamp": null, "local_common_base_id": null, "remote_common_base_id": null, "average_transfer_rate": null, "current_transfer_rate": null, "progress_percentage": null, "estimated_completion_timestamp": null, "previous_state": null, "migration_session_id": null, "is_non_disruptive": false, "is_internal": false, "replication_rule_id": "17f1c608-bd26-4d80-a818-0584af77586f", "last_sync_duration": null, "next_sync_timestamp": null, "storage_element_pairs": [{"storage_element_type": "file_system", "replication_shadow_id": null, "local_storage_element_id": "6987206a-95e2-ee30-78ac-eebf62ba21c6", "remote_storage_element_id": "69872124-1cdb-ae0e-99d9-f64cacda3004"}, {"storage_element_type": "file_system", "replication_shadow_id": null, "local_storage_element_id": "69872073-2cf5-12d4-28b4-eebf62ba21c6", "remote_storage_element_id": "69872125-709d-fd73-8be0-f64cacda3004"}, {"storage_element_type": "file_system", "replication_shadow_id": null, "local_storage_element_id": "6987207d-1729-7336-4627-eebf62ba21c6", "remote_storage_element_id": "69872125-35d6-e7c4-6f31-f64cacda3004"}, {"storage_element_type": "file_system", "replication_shadow_id": null, "local_storage_element_id": "69872087-bc02-28fa-dfa2-eebf62ba21c6", "remote_storage_element_id": "69872125-dca7-834a-fdaf-f64cacda3004"}], "failover_test_in_progress": false, "failover_test_commonbase_copy_signature": null, "sync_request_id": null, "sync_request_recoverable": true, "error_code": null, "rpo_snapshot_id": null, "data_connection_state": "OK", "parent_replication_session_id": null, "local_resource_state": "System_Defined", "witness_details": {"state": "Engaged", "witness_id": "7a892635-45c3-4500-b7f3-e03f7f6ad999", "witness_name": "witness_121", "witness_uuid": "1bdbdd70-b373-4c90-a9c6-46538d930941"}, "is_ready_for_failback": false, "system_demoted_reason": null, "auto_failover_state": "Enabled_For_Witness_Interaction", "polarization_timestamp": null, "last_witness_update_timestamp": null, "state_l10n": "OK", "role_l10n": "Destination", "resource_type_l10n": "NAS Server", "data_transfer_state_l10n": "Synchronous", "type_l10n": "Synchronous", "previous_state_l10n": null, "data_connection_state_l10n": "OK", "local_resource_state_l10n": "System Defined", "system_demoted_reason_l10n": null, "auto_failover_state_l10n": "Enabled For Witness Interaction", "remote_system": {"id": "395c39c8-2d92-43b7-a08f-b9459f785b39"}, "migration_session": null, "replication_rule": {"id": "17f1c608-bd26-4d80-a818-0584af77586f"}, "volumes": [], "volume_groups": [], "volume_group_list_cma_views": [], "volume_group_details_cma_views": []}`

      const result = detectJson(rawMessage)

      expect(result.success).toBe(true)
      expect(result.parsed).toEqual({
        id: "69872124-5668-59d5-2e21-eebf62ba21c6",
        state: "OK",
        role: "Destination",
        resource_type: "nas_server",
        data_transfer_state: "Synchronous",
        type: "Synchronous",
        last_sync_timestamp: null,
        local_resource_id: "69872124-e0ba-c439-acf8-eebf62ba21c6",
        remote_resource_id: "6987203e-3aac-ca7a-477d-eebf62ba21c6",
        remote_system_id: "395c39c8-2d92-43b7-a08f-b9459f785b39",
        failover_snapshot_id: null,
        failover_snapshot_expiration_timestamp: null,
        local_common_base_id: null,
        remote_common_base_id: null,
        average_transfer_rate: null,
        current_transfer_rate: null,
        progress_percentage: null,
        estimated_completion_timestamp: null,
        previous_state: null,
        migration_session_id: null,
        is_non_disruptive: false,
        is_internal: false,
        replication_rule_id: "17f1c608-bd26-4d80-a818-0584af77586f",
        last_sync_duration: null,
        next_sync_timestamp: null,
        storage_element_pairs: [
          {
            storage_element_type: "file_system",
            replication_shadow_id: null,
            local_storage_element_id: "6987206a-95e2-ee30-78ac-eebf62ba21c6",
            remote_storage_element_id: "69872124-1cdb-ae0e-99d9-f64cacda3004"
          },
          {
            storage_element_type: "file_system",
            replication_shadow_id: null,
            local_storage_element_id: "69872073-2cf5-12d4-28b4-eebf62ba21c6",
            remote_storage_element_id: "69872125-709d-fd73-8be0-f64cacda3004"
          },
          {
            storage_element_type: "file_system",
            replication_shadow_id: null,
            local_storage_element_id: "6987207d-1729-7336-4627-eebf62ba21c6",
            remote_storage_element_id: "69872125-35d6-e7c4-6f31-f64cacda3004"
          },
          {
            storage_element_type: "file_system",
            replication_shadow_id: null,
            local_storage_element_id: "69872087-bc02-28fa-dfa2-eebf62ba21c6",
            remote_storage_element_id: "69872125-dca7-834a-fdaf-f64cacda3004"
          }
        ],
        failover_test_in_progress: false,
        failover_test_commonbase_copy_signature: null,
        sync_request_id: null,
        sync_request_recoverable: true,
        error_code: null,
        rpo_snapshot_id: null,
        data_connection_state: "OK",
        parent_replication_session_id: null,
        local_resource_state: "System_Defined",
        witness_details: {
          state: "Engaged",
          witness_id: "7a892635-45c3-4500-b7f3-e03f7f6ad999",
          witness_name: "witness_121",
          witness_uuid: "1bdbdd70-b373-4c90-a9c6-46538d930941"
        },
        is_ready_for_failback: false,
        system_demoted_reason: null,
        auto_failover_state: "Enabled_For_Witness_Interaction",
        polarization_timestamp: null,
        last_witness_update_timestamp: null,
        state_l10n: "OK",
        role_l10n: "Destination",
        resource_type_l10n: "NAS Server",
        data_transfer_state_l10n: "Synchronous",
        type_l10n: "Synchronous",
        previous_state_l10n: null,
        data_connection_state_l10n: "OK",
        local_resource_state_l10n: "System Defined",
        system_demoted_reason_l10n: null,
        auto_failover_state_l10n: "Enabled For Witness Interaction",
        remote_system: { id: "395c39c8-2d92-43b7-a08f-b9459f785b39" },
        migration_session: null,
        replication_rule: { id: "17f1c608-bd26-4d80-a818-0584af77586f" },
        volumes: [],
        volume_groups: [],
        volume_group_list_cma_views: [],
        volume_group_details_cma_views: []
      })
    })
  })
})
