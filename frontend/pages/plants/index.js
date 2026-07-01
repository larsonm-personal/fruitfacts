import React from 'react';
import Link from 'next/link';
import Head from 'next/head';
import { getTypes } from '../../components/getTypes';

export async function getStaticProps() {
  const types = getTypes();

  return {
    props: {
      types
    }
  };
}

export default function Home({ types, setErrorMessage, setContributingLinks }) {
  React.useEffect(() => {
    setContributingLinks([
      { link: `/frontend/pages/plants/index.js`, description: `plants/index.js` },
      { link: `/plant_database/types.json5`, description: `plant types` },
      { link: `/frontend/public/fruit_icons/`, description: `fruit icons` }
    ]);
  }, []);

  React.useEffect(() => {
    setErrorMessage(null);
  }, [setErrorMessage]);

  return (
    <>
      <Head>
        <title>{`Plant Categories`}</title>
      </Head>
      <article className="prose m-5 flex max-w-none flex-col items-center">
        {types && (
          <>
            <h2>Plant Types</h2>
            {types.map((group, index) => (
              <div key={index}>
                <div>
                  <h3>{group.group_name}</h3>
                  <div key={index} className="grid grid-flow-row-dense grid-cols-4">
                    {group.types.map((type, index) => (
                      <div key={index} className="flex flex-col items-center">
                        <Link
                          href={`search?searchType=base&type=${type.name}&page=1&perPage=50&orderBy=name_then_type&order=asc`}
                          legacyBehavior
                        >
                          <img
                            className="mx-2 my-0 inline h-24 w-24 object-contain"
                            src={'/fruit_icons/' + type.name + '.svg'}
                          />
                        </Link>
                        <Link
                          href={`search?searchType=base&type=${type.name}&page=1&perPage=50&orderBy=name_then_type&order=asc`}
                          legacyBehavior
                        >
                          {type.name}
                        </Link>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </>
        )}
      </article>
    </>
  );
}
