// ideally we could have one file to render both directory listings and collections like this:
// for a path ending in "/" like /collections/Oregon/ treat this like a folder listing
// for a path ending in something else like /collections/Oregon/USDA-OSU Releases, treat it as an individual collection display

// but (Jan 2022) this isn't possible in next.js because of dumb redirect rules (all URLs get rewritten to either end in '/' or not)
// see https://github.com/vercel/next.js/discussions/23988

// so we have this split between /dirs/[...path].js (directory listings) and /collections/[...path].js (individual collections)
import React from 'react';
import Link from 'next/link';
import Head from 'next/head';
import Chart from '../../components/chart';
import { getThumbnailLocation } from '../../components/functions';
import { name_to_path, path_to_name } from '../../components/util';
import Image from 'next/image';
import { getCookie } from 'cookies-next';
import { getServerBackendBase } from '../../components/backendUrl';

export async function getServerSideProps(context) {
  let errorMessage = null;
  let { path, loc } = context.query;
  const pathJoined = path_to_name(path.join('/'));
  let location_number = parseInt(loc);
  if (isNaN(location_number)) {
    location_number = 1;
  }

  let apiURL;
  let userList = false;
  let userFromPath = null;
  const backendBase = getServerBackendBase();
  if (path[0] == 'user') {
    // incoming path will be like "user/[user name or ID]/[list name or ID]"
    // IDs are formatted like "id:123"
    userList = true;
    userFromPath = path[1];
    apiURL = `${backendBase}/api/search?searchType=loc&user=${userFromPath}&location=${path_to_name(
      path[2]
    )}`;
  } else {
    // incoming path will be like "Oregon/u-pick A"
    apiURL = `${backendBase}/api/collections/${pathJoined}`; // no trailing slash - individual collection
  }

  // todo - switch to the built-in next component when this issue is fixed:
  // https://github.com/vercel/next.js/issues/45371
  const session = getCookie('session', { req: context.req, res: context.res });
  const data = await fetch(apiURL, {
    headers: {
      cookie: `session=${session}`
    }
  })
    .then(async (response) => {
      if (response.status !== 200) {
        const text = await response.text();
        errorMessage = `backend API error: ${text}`;
        console.log(response.status + ': ' + text);
        return { items: [], locations: [] };
      } else {
        return response.json();
      }
    })
    .catch((error) => {
      errorMessage = `can't reach backend: ${error.message}`;
      console.log(error);
      return { items: [], locations: [] };
    });

  // cut down data to only this location or location #1 if not specified
  data.items = data.items
    ? data.items.filter((item) => {
        return item.location_number == location_number;
      })
    : [];

  let location;
  if (data?.locations?.length) {
    location = data.locations.find((location) => {
      return location.location_number == location_number;
    }) || { location_name: `unknown location #${location_number}` };
  } else {
    location = 0;
  }

  const thumbnail = getThumbnailLocation(`${pathJoined}.jpg`);

  return {
    props: {
      data,
      location,
      userList,
      userFromPath,
      pathJoined,
      thumbnail,
      errorMessage
    }
  };
}

// github link example
// plant_database/references/Oregon/2017%20-%20Table%20Grape%20Cultivar%20Performance%20in%20Oregon's%20Willamette%20Valley.json5

export default function Home({
  data,
  location,
  userList,
  userFromPath,
  pathJoined,
  thumbnail,
  errorMessage,
  setErrorMessage,
  setContributingLinks
}) {
  let data_link = `plant_database/references/${pathJoined}.json5`;
  React.useEffect(() => {
    setContributingLinks([
      {
        link: data_link,
        description: `data for this collection`
      },
      {
        link: `/frontend/pages/collections/[...path].js`,
        description: `collections/[...path].js`
      },
      { link: `/frontend/components/chart/`, description: `chart component` }
    ]);
  }, []);

  setErrorMessage(errorMessage);
  return (
    <>
      <article className="prose m-5 max-w-none">
        <Head>
          <title>{`Collection: ${pathJoined}`}</title>
        </Head>
        {userList && (
          <h1>
            {data.user_name}&apos;s location &quot;{data.locations[0]?.location_name}&quot;
          </h1>
        )}
        {!userList && (
          <Image src={thumbnail} alt="preview image for this reference" width={200} height={200} />
        )}
        {data.collection && (
          <>
            {data.collection.needs_help == 1 && (
              <p>
                {'the data for this collection is marked "needs help". '}
                <a href={`${process.env.NEXT_PUBLIC_GITHUB_BASE}${data_link}`}>
                  click here to view it on github
                </a>
              </p>
            )}
            <p>
              {`${data.collection.title} `}
              {data.collection.url && <a href={data.collection.url}>[ref]</a>}
            </p>
            <h1>Locations</h1>
            <ul className="list-disc">
              {data?.locations?.length > 1 ? (
                <>
                  {data.locations.map((location) => (
                    <li key={location.id}>
                      <Link
                        href={`/collections/${name_to_path(pathJoined)}?loc=${
                          location.location_number
                        }`}
                        legacyBehavior
                      >
                        {location.location_name}
                      </Link>
                    </li>
                  ))}
                </>
              ) : (
                <>
                  {data.locations.map((location) => (
                    <li key={location.id}>{location.location_name}</li>
                  ))}
                </>
              )}
            </ul>
            {data?.locations?.length > 1 ? (
              <h1>{`Chart (${location.location_name})`}</h1>
            ) : (
              <h1>Chart</h1>
            )}
            <div className="border-2 border-solid">
              <Chart items={data.items} />
            </div>
          </>
        )}
        {data?.locations?.length > 1 ? (
          <h1>{`Plants (${location.location_name})`}</h1>
        ) : (
          <h1>Plants</h1>
        )}
        {userList && (
          <Link
            href={`/search?searchType=base&user=${userFromPath}&addToList=${data.locations[0]?.location_name}`}
            legacyBehavior
          >
            <p>add</p>
          </Link>
        )}
        <ul className="list-none">
          {data?.items?.length > 1 ? (
            data.items.map((item) => (
              <li key={item.id}>
                <img className="" src={'/fruit_icons/' + item.type + '.svg'} />
                <Link href={`/plant/${name_to_path(item.type + '/' + item.name)}`} legacyBehavior>
                  {item.name + ' ' + item.type}
                </Link>
                {item.marketing_name && <> (marketed under the {item.marketing_name} brand)</>}
              </li>
            ))
          ) : (
            <li>no plants</li>
          )}
        </ul>
      </article>
    </>
  );
}
