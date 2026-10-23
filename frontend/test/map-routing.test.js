/** @jest-environment jsdom */
import React, { act } from 'react';
import { createRoot } from 'react-dom/client';
import Directory from '../pages/dirs/[[...path]]';

let mockMapProps;
let mockRouter;
jest.mock('next/router', () => ({ useRouter: () => mockRouter }));
jest.mock('next/head', () => ({ __esModule: true, default: () => null }));
jest.mock('next/dynamic', () => ({
  __esModule: true,
  default: () =>
    function Map(props) {
      mockMapProps = props;
      return null;
    }
}));

global.IS_REACT_ACT_ENVIRONMENT = true;
let root;
let container;
let error;
let props;
beforeEach(() => {
  container = document.createElement('div');
  root = createRoot(container);
  error = jest.fn();
  props = {
    data: {},
    pathUsed: 'Oregon/',
    errorMessage: null,
    setErrorMessage: error,
    setContributingLinks: jest.fn()
  };
  mockRouter = {
    asPath: '/dirs/Oregon/',
    push: jest.fn((asPath) => {
      mockRouter = { ...mockRouter, asPath };
    })
  };
  global.fetch = jest.fn().mockResolvedValue({ status: 200, json: async () => [] });
  jest.spyOn(console, 'log').mockImplementation(() => {});
});
afterEach(async () => {
  await act(async () => root.unmount());
  container.remove();
  jest.restoreAllMocks();
  delete global.fetch;
});

test('map sync keeps its directory and does not push again when the router changes', async () => {
  await act(async () => root.render(<Directory {...props} />));
  await act(async () => {
    mockMapProps.setCenterForQuery({ lat: 0, lng: 0 });
    mockMapProps.setZoomForQuery(4);
  });
  expect(mockRouter.push).toHaveBeenCalledWith('/dirs/Oregon/@0.0000000,0.000000,4z', undefined, {
    shallow: true,
    scroll: false
  });
  await act(async () => root.render(<Directory {...props} />));
  expect(mockRouter.push).toHaveBeenCalledTimes(1);
});

test('map fetch failures reach the displayed error setter', async () => {
  fetch.mockRejectedValueOnce(new Error('fixture offline'));
  await act(async () => root.render(<Directory {...props} />));
  expect(error).toHaveBeenCalledWith("can't reach backend: fixture offline");
});
