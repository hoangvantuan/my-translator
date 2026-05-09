import sys
import threading
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from scripts.local_pipeline import LocalPipeline


def make_pipeline_with_buffer(data: bytes):
    pipeline = LocalPipeline.__new__(LocalPipeline)
    pipeline.audio_buffer = bytearray(data)
    pipeline.lock = threading.Lock()
    return pipeline


def test_compact_audio_buffer_discards_processed_prefix():
    pipeline = make_pipeline_with_buffer(b"abcdefghij")
    new_pos = pipeline._compact_audio_buffer(4)
    assert new_pos == 0
    assert bytes(pipeline.audio_buffer) == b"efghij"


def test_compact_audio_buffer_keeps_buffer_when_nothing_processed():
    pipeline = make_pipeline_with_buffer(b"abcdefghij")
    new_pos = pipeline._compact_audio_buffer(0)
    assert new_pos == 0
    assert bytes(pipeline.audio_buffer) == b"abcdefghij"


def test_compact_audio_buffer_clears_when_processed_past_end():
    pipeline = make_pipeline_with_buffer(b"abc")
    new_pos = pipeline._compact_audio_buffer(10)
    assert new_pos == 0
    assert bytes(pipeline.audio_buffer) == b""
