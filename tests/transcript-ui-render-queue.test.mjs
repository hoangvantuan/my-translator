import assert from 'node:assert/strict';
import test from 'node:test';
import { JSDOM } from 'jsdom';
import { TranscriptUI } from '../src/js/ui.js';

function setupDom() {
    const dom = new JSDOM('<!doctype html><body><div id="overlay-view"></div></body>');
    global.document = dom.window.document;
    global.window = dom.window;

    const callbacks = [];
    global.requestAnimationFrame = (callback) => {
        callbacks.push(callback);
        return callbacks.length;
    };

    const scrollHost = document.createElement('div');
    const container = document.createElement('div');
    scrollHost.appendChild(container);
    document.body.appendChild(scrollHost);

    return { container, callbacks };
}

test('TranscriptUI renders at most once per animation frame', () => {
    const { container, callbacks } = setupDom();
    const ui = new TranscriptUI(container);

    let renderCount = 0;
    const originalRenderNow = ui._renderNow.bind(ui);
    ui._renderNow = () => {
        renderCount += 1;
        originalRenderNow();
    };

    ui.addOriginal('こんにちは');
    ui.addTranslation('Xin chào');
    ui.setProvisional('テスト');

    assert.equal(renderCount, 0);
    assert.equal(callbacks.length, 1);

    callbacks.shift()();

    assert.equal(renderCount, 1);
    assert.match(container.innerHTML, /Xin chào/);
});

test('TranscriptUI cancels stale scheduled render after clear', () => {
    const { container, callbacks } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('古いテキスト');
    ui.clear();

    assert.equal(container.innerHTML, '');
    assert.equal(callbacks.length, 1);

    callbacks.shift()();

    assert.equal(container.innerHTML, '');
});
