#!/usr/bin/env python3
"""Regression checks for response ownership in the verification LSP client."""

from __future__ import annotations

import unittest
from unittest.mock import Mock

from lsp_protocol import LspClient, LspProtocolError


def response(request_id: int | str, result: object = None) -> dict:
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


class ResponseOwnershipTests(unittest.TestCase):
    def setUp(self) -> None:
        self.client = object.__new__(LspClient)
        self.client.timeout = 1.0
        self.client.next_id = 1
        self.client.notifications = []
        self.client.responses = {}
        self.client.pending_requests = set()
        self.client._send = Mock()

    def incoming(self, *messages: dict) -> None:
        self.client._read_message = Mock(side_effect=messages)

    def test_unknown_response_is_rejected(self) -> None:
        self.client.send_request(1, "test")
        self.incoming(response(99), response(1))
        with self.assertRaisesRegex(LspProtocolError, "unexpected or duplicate"):
            self.client.wait_for_response(1)

    def test_duplicate_queued_response_is_rejected(self) -> None:
        self.client.send_request(1, "first")
        self.client.send_request(2, "second")
        self.incoming(response(2), response(2), response(1))
        with self.assertRaisesRegex(LspProtocolError, "unexpected or duplicate"):
            self.client.wait_for_response(1)

    def test_duplicate_consumed_response_is_rejected(self) -> None:
        self.client.send_request(1, "first")
        self.incoming(response(1))
        self.client.wait_for_response(1)
        self.client.send_request(2, "second")
        self.incoming(response(1), response(2))
        with self.assertRaisesRegex(LspProtocolError, "unexpected or duplicate"):
            self.client.wait_for_response(2)

    def test_matching_id_server_request_is_rejected(self) -> None:
        self.client.send_request(1, "test")
        self.incoming({"jsonrpc": "2.0", "id": 1, "method": "server/request"})
        with self.assertRaisesRegex(LspProtocolError, "unexpected server request"):
            self.client.wait_for_response(1)

    def test_out_of_order_responses_remain_owned(self) -> None:
        self.client.send_request(1, "first")
        self.client.send_request("second", "second")
        self.incoming(response("second", "b"), response(1, "a"))
        self.assertEqual(self.client.wait_for_response(1)["result"], "a")
        self.assertEqual(self.client.wait_for_response("second")["result"], "b")
        self.assertFalse(self.client.pending_requests)

    def test_response_survives_notification_wait(self) -> None:
        self.client.send_request(1, "test")
        notification = {"jsonrpc": "2.0", "method": "ready", "params": {}}
        self.incoming(response(1, "saved"), notification)
        self.assertEqual(self.client.wait_for_notification("ready"), notification)
        self.assertEqual(self.client.wait_for_response(1)["result"], "saved")

    def test_notification_wait_rejects_unexpected_response(self) -> None:
        self.incoming(response(99))
        with self.assertRaisesRegex(LspProtocolError, "unexpected or duplicate"):
            self.client.wait_for_notification("ready")

    def test_malformed_responses_are_rejected(self) -> None:
        self.client.send_request(1, "test")
        for message in (
            {"jsonrpc": "2.0", "id": 1},
            {"jsonrpc": "2.0", "id": 1, "result": None, "error": {}},
            response(True),
            response(None),
            {"id": 1, "result": None},
        ):
            with self.subTest(message=message):
                with self.assertRaises(LspProtocolError):
                    self.client._accept_message(message)
                self.assertIn(1, self.client.pending_requests)

    def test_request_id_cannot_be_reused_until_response_consumed(self) -> None:
        self.client.send_request(1, "test")
        with self.assertRaisesRegex(LspProtocolError, "already outstanding"):
            self.client.send_request(1, "test")
        self.client._accept_message(response(1))
        with self.assertRaisesRegex(LspProtocolError, "already outstanding"):
            self.client.send_request(1, "test")
        self.client.wait_for_response(1)
        self.client.send_request(1, "test")
        self.assertIn(1, self.client.pending_requests)

    def test_unissued_wait_fails_immediately(self) -> None:
        with self.assertRaisesRegex(LspProtocolError, "was not issued"):
            self.client.wait_for_response(1)

    def test_close_accepts_pipe_closed_after_successful_exit(self) -> None:
        self.client.process = Mock()
        self.client.process.stdin.close.side_effect = BrokenPipeError
        self.client.process.returncode = 0
        self.client.close()
        self.client.process.wait.assert_called_once_with(timeout=10)

    def test_close_still_rejects_server_failure_with_closed_pipe(self) -> None:
        self.client.process = Mock()
        self.client.process.stdin.close.side_effect = BrokenPipeError
        self.client.process.returncode = 1
        self.client.process.stderr.read.return_value = b"server failed"
        self.client._diagnostic_context = Mock(return_value="server failed")
        with self.assertRaisesRegex(LspProtocolError, "server failed"):
            self.client.close()

    def test_cancellation_preserves_wire_error(self) -> None:
        error = {"code": -32800, "message": "cancelled"}
        self.incoming({"jsonrpc": "2.0", "id": 1, "error": error})
        self.assertEqual(self.client.request_error("test"), error)

    def test_request_reports_cancellation_as_failure(self) -> None:
        self.incoming({"jsonrpc": "2.0", "id": 1, "error": {"code": -32800}})
        with self.assertRaisesRegex(LspProtocolError, "-32800"):
            self.client.request("test")


if __name__ == "__main__":
    unittest.main()
