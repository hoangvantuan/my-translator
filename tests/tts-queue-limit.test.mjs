import assert from 'node:assert/strict';
import test from 'node:test';

global.window = {
    __TAURI__: {
        core: {
            invoke: async () => 'base64-audio',
        },
    },
};

test('Edge TTS drops oldest queued text when queue is full', async () => {
    const { edgeTTSRust } = await import(`../src/js/edge-tts.js?edge=${Date.now()}`);
    edgeTTSRust.disconnect();
    edgeTTSRust._isSpeaking = true;

    for (let i = 0; i < 8; i++) {
        edgeTTSRust.speak(`edge ${i}`);
    }

    assert.equal(edgeTTSRust._queue.length, 5);
    assert.deepEqual(edgeTTSRust._queue, ['edge 3', 'edge 4', 'edge 5', 'edge 6', 'edge 7']);
});

test('Google TTS drops oldest queued text when queue is full', async () => {
    const { googleTTS } = await import(`../src/js/google-tts.js?google=${Date.now()}`);
    googleTTS.disconnect();
    googleTTS._isSpeaking = true;

    for (let i = 0; i < 8; i++) {
        googleTTS.speak(`google ${i}`);
    }

    assert.equal(googleTTS._queue.length, 5);
    assert.deepEqual(googleTTS._queue, ['google 3', 'google 4', 'google 5', 'google 6', 'google 7']);
});

test('ElevenLabs TTS limits queued text before websocket connects', async () => {
    const { elevenLabsTTS } = await import(`../src/js/elevenlabs-tts.js?eleven=${Date.now()}`);
    elevenLabsTTS.disconnect();
    elevenLabsTTS.ws = { readyState: WebSocket.CLOSED };
    elevenLabsTTS.connect = () => {};

    for (let i = 0; i < 12; i++) {
        elevenLabsTTS.speak(`eleven ${i}`);
    }

    assert.equal(elevenLabsTTS._textQueue.length, 10);
    assert.deepEqual(elevenLabsTTS._textQueue[0], 'eleven 2');
    assert.deepEqual(elevenLabsTTS._textQueue[9], 'eleven 11');
});
