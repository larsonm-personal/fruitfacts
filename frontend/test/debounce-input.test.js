/** @jest-environment jsdom */
import React, { act } from 'react';
import { createRoot } from 'react-dom/client';
import DebounceInput from '../components/debounceInput';

global.IS_REACT_ACT_ENVIRONMENT = true;
let container;
let root;
let changed;

beforeEach(() => {
  jest.useFakeTimers();
  container = document.createElement('div');
  document.body.appendChild(container);
  root = createRoot(container);
  changed = jest.fn();
  act(() => root.render(<DebounceInput minLength={2} debounceTimeout={300} onChange={changed} />));
});
afterEach(() => {
  act(() => root.unmount());
  container.remove();
  jest.useRealTimers();
});
function type(value) {
  const input = container.querySelector('input');
  act(() => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(input, value);
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
}

test('publishes only the latest search after the debounce delay', () => {
  type('ap');
  act(() => jest.advanceTimersByTime(200));
  type('apple');
  act(() => jest.advanceTimersByTime(299));
  expect(changed).not.toHaveBeenCalled();
  act(() => jest.advanceTimersByTime(1));
  expect(changed).toHaveBeenCalledTimes(1);
  expect(changed.mock.calls[0][0].target.value).toBe('apple');
});
test('short input cancels pending searches and clearing resets the filter', () => {
  type('ap');
  type('a');
  act(() => jest.advanceTimersByTime(300));
  expect(changed).not.toHaveBeenCalled();
  type('');
  act(() => jest.advanceTimersByTime(300));
  expect(changed.mock.calls[0][0].target.value).toBe('');
});
test('unmount cancels pending searches', () => {
  type('apple');
  act(() => root.render(null));
  act(() => jest.advanceTimersByTime(300));
  expect(changed).not.toHaveBeenCalled();
});
