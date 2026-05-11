import assert from 'node:assert/strict';
import test from 'node:test';
import { JSDOM } from 'jsdom';
import { TranscriptUI } from '../src/js/ui.js';

function setupDom() {
    const dom = new JSDOM('<!doctype html><body><div id="overlay-view"></div></body>');
    global.document = dom.window.document;
    global.window = dom.window;
    global.requestAnimationFrame = (cb) => { cb(); return 1; };

    const scrollHost = document.createElement('div');
    const container = document.createElement('div');
    scrollHost.appendChild(container);
    document.body.appendChild(scrollHost);
    return { container };
}

test('addTranslation matches existing original segment (speaker already set)', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('こんにちは', '1', 'ja');
    ui.addTranslation('Hello', '1', 'ja');

    assert.equal(ui.segments.length, 1);
    assert.equal(ui.segments[0].translation, 'Hello');
    assert.equal(ui.segments[0].speaker, '1');
    assert.equal(ui.segments[0].language, 'ja');
    assert.equal(ui.segments[0].status, 'translated');
});

test('addTranslation creates new segment with speaker/language when no original match', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addTranslation('Orphan translation', '2', 'en');

    assert.equal(ui.segments.length, 1);
    assert.equal(ui.segments[0].speaker, '2');
    assert.equal(ui.segments[0].language, 'en');
    assert.equal(ui.segments[0].translation, 'Orphan translation');
    assert.equal(ui.segments[0].original, '');
});

test('addTranslation without speaker/language still works (backward compat)', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('テスト', '1', 'ja');
    ui.addTranslation('Test');

    assert.equal(ui.segments[0].translation, 'Test');
    assert.equal(ui.segments[0].speaker, '1');
});

test('_speakerIndex assigns sequential indices starting from 1', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    assert.equal(ui._speakerIndex('1'), 1);
    assert.equal(ui._speakerIndex('2'), 2);
    assert.equal(ui._speakerIndex('1'), 1);
    assert.equal(ui._speakerIndex('3'), 3);
    assert.equal(ui._speakerIndex('4'), 4);
    assert.equal(ui._speakerIndex('5'), 'default');
});

test('_speakerIndex resets after clear()', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui._speakerIndex('1');
    ui._speakerIndex('2');
    ui.clear();

    assert.equal(ui._speakerIndex('1'), 1);
});

test('_speakerIndex resets after showPlaceholder()', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui._speakerIndex('1');
    ui._speakerIndex('2');
    ui.showPlaceholder();

    assert.equal(ui._speakerIndex('3'), 1);
});
