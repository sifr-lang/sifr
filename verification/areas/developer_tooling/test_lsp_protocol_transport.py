#!/usr/bin/env python3
"""Compiler-independent framing and cleanup regressions."""
import json
import os
import shlex
import sys
import time
import unittest
from unittest.mock import patch

from lsp_protocol import LspClient, LspProtocolError

MESSAGE = {"jsonrpc": "2.0", "method": "fixture/notification", "params": {}}
BODY = json.dumps(MESSAGE).encode()
FRAME = f"Content-Length: {len(BODY)}\r\n\r\n".encode() + BODY


class TransportTests(unittest.TestCase):
    def client(self, code):
        command = shlex.join([sys.executable, "-c", code])
        with patch.dict(os.environ, {"SIFR_LSP_COMMAND": command}):
            client = LspClient(timeout=1)
        self.addCleanup(self.reap, client)
        return client

    @staticmethod
    def reap(client):
        if client.process.poll() is None:
            client.process.kill()
        client.process.wait(timeout=5)
        for stream in (client.process.stdin, client.process.stdout, client.process.stderr):
            stream.close()

    def test_coalesced_frames_while_producer_remains_alive(self):
        client = self.client(f"import os,time; os.write(1,{FRAME + FRAME!r}); time.sleep(10)")
        deadline = time.monotonic() + 2
        self.assertEqual(client._read_message(deadline), MESSAGE)
        self.assertEqual(client._read_message(deadline), MESSAGE)
        self.assertIsNone(client.process.poll())

    def test_fragmented_header_and_body(self):
        pieces = [FRAME[:8], FRAME[8:25], FRAME[25:-5], FRAME[-5:]]
        client = self.client(f"import os,time; [(os.write(1,part),time.sleep(0.03)) for part in {pieces!r}]; time.sleep(10)")
        self.assertEqual(client._read_message(time.monotonic() + 2), MESSAGE)

    def test_partial_header_and_body_obey_deadline(self):
        for prefix in (FRAME[:8], FRAME[:-5]):
            with self.subTest(prefix=prefix):
                client = self.client(f"import os,time; os.write(1,{prefix!r}); time.sleep(10)")
                started = time.monotonic()
                with self.assertRaisesRegex(LspProtocolError, "timed out"):
                    client._read_message(started + 0.3)
                self.assertLess(time.monotonic() - started, 2)

    def test_eof_during_header_and_body(self):
        for prefix, stage in ((FRAME[:8], "header"), (FRAME[:-5], "body")):
            with self.subTest(stage=stage):
                client = self.client(f"import os; os.write(1,{prefix!r})")
                with self.assertRaisesRegex(LspProtocolError, f"reading {stage}"):
                    client._read_message(time.monotonic() + 2)

    def test_complete_frames_remain_readable_after_exit(self):
        client = self.client(f"import os; os.write(1,{FRAME + FRAME!r})")
        client.process.wait(timeout=5)
        self.assertEqual(client._read_message(time.monotonic() + 2), MESSAGE)
        self.assertEqual(client._read_message(time.monotonic() + 2), MESSAGE)

    def test_cleanup_preserves_primary_failure(self):
        client = self.client("import sys; sys.exit(1)")
        client.process.wait(timeout=5)
        primary = LspProtocolError("primary timeout")
        try:
            try:
                raise primary
            finally:
                client.close()
        except LspProtocolError as error:
            self.assertIs(error, primary)
            self.assertIn("LSP exited 1", error.__notes__[0])
        else:
            self.fail("primary error was suppressed")

    def test_cleanup_failure_without_primary_is_raised(self):
        client = self.client("import sys; sys.exit(1)")
        client.process.wait(timeout=5)
        with self.assertRaisesRegex(LspProtocolError, "LSP exited 1"):
            client.close()


if __name__ == "__main__":
    unittest.main()
