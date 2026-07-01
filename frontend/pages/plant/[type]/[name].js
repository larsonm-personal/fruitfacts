import React from 'react';
import Link from 'next/link';
import Head from 'next/head';
import { formatPatentDate } from '../../../components/functions';
import { name_to_path, path_to_name } from '../../../components/util';
import { getServerBackendBase } from '../../../components/backendUrl';

export async function getServerSideProps(context) {
  let errorMessage = null;
  let { type, name } = context.query;
  type = path_to_name(type);
  name = path_to_name(name);
  const backendBase = getServerBackendBase();
  const plant = await fetch(`${backendBase}/api/plants/${name_to_path(type + '/' + name)}`)
    .then((response) => {
      if (response.status !== 200) {
        return response.text().then((text) => {
          errorMessage = `backend API error: ${text}`;
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

  return {
    props: {
      plant,
      type,
      name,
      errorMessage
    }
  };
}

export default function Home({
  plant,
  type,
  name,
  errorMessage,
  setErrorMessage,
  setContributingLinks
}) {
  React.useEffect(() => {
    setContributingLinks([
      { link: `/frontend/pages/plant/[type]/[name].js`, description: `plant/[type]/[name].js` }
    ]);
  }, []);

  React.useEffect(() => {
    setErrorMessage(errorMessage);
  }, [errorMessage, setErrorMessage]);

  return (
    <>
      <Head>
        <title>{`${name} ${type}`}</title>
      </Head>
      <article className="prose m-5">
        {plant.base && (
          <h2>
            <img
              className="mx-2 my-0 inline h-24 w-24 object-contain"
              src={'/fruit_icons/' + plant.base.type + '.svg'}
            />
            {plant.base.name} {plant.base.type}
          </h2>
        )}
        {plant.base?.marketing_name && (
          <h2> marketed under the {plant.base.marketing_name} brand </h2>
        )}
        <p>
          {plant.base?.uspp_number && <>USPP{plant.base.uspp_number} </>}

          {plant.base?.uspp_expiration && (
            <>
              until{' '}
              {formatPatentDate(plant.base.uspp_expiration, plant.base.uspp_expiration_estimated)}
            </>
          )}

          {plant.base?.uspp_number && (
            <>
              {' '}
              <a href={`https://patents.google.com/patent/USPP${plant.base.uspp_number}`}>
                google patents
              </a>
            </>
          )}
        </p>
        {plant.base?.aka && <p>AKA {plant.base.aka}</p>}

        {(plant.base?.release_year || plant.base?.released_by) && (
          <p>
            released
            {plant.base?.release_year && <> {plant.base.release_year}</>}
            {plant.base?.released_by && <> by {plant.base.released_by}</>}
          </p>
        )}

        {plant.base?.harvest_relative_to && plant.base?.harvest_relative != undefined && (
          <p>
            {`${plant.base.harvest_relative_to} ${plant.base.harvest_relative >= 0 ? '+' : ''}${
              plant.base.harvest_relative
            }`}
          </p>
        )}

        {plant.base?.s_allele && <p>{`S-Allele: ${plant.base?.s_allele}`}</p>}

        {plant.collection && (
          <>
            {' '}
            <h1>Collections</h1>
            <ul className="list-disc">
              {plant.collection.map((entry) => (
                <>
                  <li>
                    <Link
                      href={`/collections/${name_to_path(entry.path_and_filename)}`}
                      legacyBehavior
                    >
                      {entry.path_and_filename}
                    </Link>
                    {entry.description && <p>{entry.description}</p>}
                  </li>
                </>
              ))}
            </ul>
          </>
        )}

        {plant.collection && (
          <>
            <h1>Harvest Times</h1>
            <ul className="list-disc">
              {plant.collection.map((entry) => (
                <>
                  {entry.harvest_text && (
                    <li>
                      {`${entry.harvest_text} `}
                      <Link
                        href={`/collections/${name_to_path(entry.path_and_filename)}`}
                        title={entry.path_and_filename}
                      >
                        [ref]
                      </Link>
                    </li>
                  )}
                </>
              ))}
            </ul>
          </>
        )}
      </article>
    </>
  );
}
