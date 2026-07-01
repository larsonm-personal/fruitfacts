import React from 'react';
import Link from 'next/link';
import Head from 'next/head';
import { format as timeAgo } from 'timeago.js';
import { name_to_path } from '../components/util';
import { getServerBackendBase } from '../components/backendUrl';

// todo:
// - flatten and sort by common name (see types.name_alphabetical)
// - sort by latin name
// - grid/mobile view

export async function getServerSideProps() {
  let errorMessage = null;
  const backendBase = getServerBackendBase();
  const fact = await fetch(`${backendBase}/api/fact`)
    .then((response) => {
      if (response.status !== 200) {
        return response.text().then((text) => {
          errorMessage = `can't reach backend: ${text}`;
          console.log(text);
          return {};
        });
      }
      return response.json();
    })
    .catch((error) => {
      errorMessage = `can't reach backend: ${error.message}`;
      console.log(error);
      return {};
    });

  const recentChangesData = await fetch(`${backendBase}/api/recent_changes`)
    .then((response) => {
      if (response.status !== 200) {
        response.text().then((text) => {
          errorMessage = `can't reach backend: ${text}`;
          console.log(text);
        });
        return {};
      }
      return response.json();
    })
    .catch((error) => {
      errorMessage = `can't reach backend: ${error.message}`;
      console.log(error);
      return {};
    });

  return {
    props: {
      fact,
      recentChangesData,
      errorMessage
    }
  };
}

export default function Home({
  fact,
  recentChangesData,
  errorMessage,
  setErrorMessage,
  setContributingLinks
}) {
  React.useEffect(() => {
    setContributingLinks([{ link: `/frontend/pages/index.js`, description: `index.js` }]);
  }, []);

  React.useEffect(() => {
    setErrorMessage(errorMessage);
  }, [errorMessage, setErrorMessage]);

  return (
    <article className="prose m-5">
      <Head>
        <title>fruitfacts</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>
      <main>
        <div>
          {fact.fact && (
            <p>
              <b>Fact:</b> {`${fact.fact} `}
              <a href={` ${fact.reference}`}>[ref]</a>
            </p>
          )}
        </div>
        <div>
          {recentChangesData.recentChanges && (
            <>
              <h3>Recent Changes</h3>
              <ul className="list-disc">
                {recentChangesData.recentChanges.collectionChanges.map((update, index) => (
                  <li key={index}>
                    <Link
                      href={`/collections/${name_to_path(update.path + update.filename)}`}
                      legacyBehavior
                    >
                      {update.filename}
                    </Link>
                    <div className="m-1 inline">{timeAgo(update.gitEditTime * 1000)}</div>
                  </li>
                ))}
              </ul>
            </>
          )}
        </div>
        <div>
          {(recentChangesData.recentChanges || recentChangesData.buildInfo) && <h3>Build Info</h3>}
          <>
            {recentChangesData.recentChanges && (
              <div className="inline">
                {recentChangesData.recentChanges.basePlantsCount} varieties in{' '}
                {recentChangesData.recentChanges.referencesCount} references.
              </div>
            )}
            {recentChangesData.buildInfo && (
              <div className="m-1 inline">
                updated {timeAgo(recentChangesData.buildInfo.gitUnixTime * 1000)}
                {', '}
                build count {recentChangesData.buildInfo.gitCommitCount},{' '}
                <a
                  href={` ${`${process.env.NEXT_PUBLIC_GITHUB_HOMEPAGE}/tree/${recentChangesData.buildInfo.gitHash}`}`}
                >
                  git hash {recentChangesData.buildInfo.gitHash.substring(0, 7)}
                </a>
              </div>
            )}
          </>
        </div>
      </main>

      <footer></footer>
    </article>
  );
}
