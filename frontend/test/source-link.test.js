import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import Home from '../pages/index';

jest.mock('next/head', () => ({ __esModule: true, default: () => null }));

test('homepage source link uses the personal fork despite stale server configuration and actual backend commit', () => {
  process.env.NEXT_PUBLIC_GITHUB_HOMEPAGE = 'https://github.com/larsonmpdx/fruitfacts';
  const hash = '7fbd65660c2ac17f51c8baf7c3b91f4cac2c8885';
  const html = renderToStaticMarkup(
    <Home
      fact={{}}
      recentChangesData={{
        buildInfo: {
          gitHash: hash,
          gitUnixTime: 1,
          gitCommitCount: 1
        }
      }}
      setErrorMessage={() => {}}
      setContributingLinks={() => {}}
    />
  );
  expect(html).toContain(`href="https://github.com/larsonm-personal/fruitfacts/tree/${hash}"`);
  expect(html).not.toContain('github.com/larsonmpdx/fruitfacts');
});
