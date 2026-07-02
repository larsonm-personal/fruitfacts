// ideally we could have one file to render both directory listings and collections like this:
// for a path ending in "/" like /collections/Oregon/ treat this like a folder listing
// for a path ending in something else like /collections/Oregon/USDA-OSU Releases, treat it as an individual collection display

// but (Jan 2022) this isn't possible in next.js because of dumb redirect rules (all URLs get rewritten to either end in '/' or not)
// see https://github.com/vercel/next.js/discussions/23988

// so we have this split between /dirs/[...path].js (directory listings) and /collections/[...path].js (individual collections)
import React from 'react';
import Link from 'next/link';
import Head from 'next/head';
import dynamic from 'next/dynamic';
import throttle from 'lodash/throttle';
import { useRouter } from 'next/router';
import { name_to_path, path_to_name } from '../../components/util';
import { getServerBackendBase } from '../../components/backendUrl';

// see https://nextjs.org/docs/advanced-features/dynamic-import
const Map = dynamic(() => import('../../components/map'), { ssr: false });
const MAP_LOCATION_LIMIT = 5000;

function isValidBounds(extents) {
  return (
    Array.isArray(extents) &&
    extents.length === 4 &&
    extents.every((value) => Number.isFinite(value))
  );
}

function getLocationsFetchError(error) {
  let message = `can't reach locations backend: ${error.message}`;
  if (typeof window !== 'undefined') {
    try {
      const backendHost = new URL(process.env.NEXT_PUBLIC_BACKEND_BASE).hostname;
      if (backendHost == 'local.fruitfacts.xyz' && window.location.hostname != backendHost) {
        message += '; open http://local.fruitfacts.xyz:3000 instead of http://localhost:3000';
      }
    } catch {
      // leave the plain fetch error
    }
  }
  return message;
}

function getDirectoryHref(directory) {
  if (directory == '/') {
    return '/dirs#dirs';
  }
  return `/dirs/${name_to_path(directory)}#dirs`;
}

export async function getServerSideProps(context) {
  const { path } = context.query;
  let pathUsed;
  let initialLocation = {};
  let errorMessage = null;
  if (path) {
    const pathCombined = path_to_name(path.join('/'));

    // remove the part after the '@' and break it down
    // will look like "...@45.1234,-123.1234,4z"
    const location_regex = /@([0-9.-]+),([0-9.-]+),([0-9]+)z/;
    const match = pathCombined.match(location_regex);
    if (match && match.length >= 4) {
      initialLocation.lat = match[1];
      initialLocation.lon = match[2];
      initialLocation.zoom = match[3];
      console.log(`match: ${JSON.stringify(initialLocation, null, 2)}`);
    }

    pathUsed = pathCombined.split('@')[0];

    console.log(`pathused 1: ${pathUsed}`);
    if (!pathUsed.endsWith('/')) {
      pathUsed += '/';
    }
    console.log(`pathused 2: ${pathUsed}`);
    pathUsed = pathUsed.replace(/(\/)\/+/g, '$1'); // remove repeated forward slashes
    console.log(`pathused 3: ${pathUsed}`);
    if (pathUsed == '/') {
      pathUsed = ''; // special case - would like to not need this but the backend wants it in the /api/collections/ call
    }
  } else {
    pathUsed = ''; // this combind with the [[...path]].js filename gets us the base path "/dirs" or "/dirs/"
  }
  const backendBase = getServerBackendBase();
  const data = await fetch(`${backendBase}/api/collections/${name_to_path(pathUsed)}`)
    .then((response) => {
      if (response.status !== 200) {
        return response.text().then((text) => {
          errorMessage = `backend API error: ${text}`;
          console.log(text);
          return [];
        });
      }
      return response.json();
    })
    .catch((error) => {
      errorMessage = `can't reach backend: ${error.message}`;
      console.log(error);
      return [];
    });

  //  console.log(JSON.stringify(data, null, 2));
  return {
    props: {
      data,
      pathUsed,
      initialLocation,
      errorMessage
    }
  };
}

export default function Home({
  data,
  pathUsed,
  initialLocation,
  errorMessage,
  setErrorMessage,
  setContributingLinks
}) {
  React.useEffect(() => {
    setContributingLinks([
      { link: `/frontend/pages/dirs/[[...path]].js`, description: `dirs/[[...path]].js` },
      { link: `/frontend/components/map/`, description: `map component` },
      { link: `/backend/src/queries/map.rs`, description: `map APIs` }
    ]);
  }, []);

  const [center, setCenterForQuery] = React.useState({});
  const [zoom, setZoomForQuery] = React.useState({});
  const [extents, setExtentsForFetch] = React.useState(null);
  const [locations, setLocations] = React.useState([]);

  React.useEffect(() => {
    setErrorMessage(errorMessage);
  }, [errorMessage, setErrorMessage]);

  const runFetchLocations = React.useMemo(
    // useMemo(): cache results for each input and don't re-run. appears to not be doing anything
    () =>
      throttle(async (extents, callback) => {
        if (!isValidBounds(extents)) {
          return;
        }
        console.log('hi' + JSON.stringify(extents));

        const response = await fetch(
          `${process.env.NEXT_PUBLIC_BACKEND_BASE}/api/locations?` +
            new URLSearchParams({
              // extents are "[minx, miny, maxx, maxy]"
              min_lon: extents[0],
              min_lat: extents[1],
              max_lon: extents[2],
              max_lat: extents[3],
              limit: MAP_LOCATION_LIMIT
            })
        )
          .then((response) => {
            if (response.status !== 200) {
              return response.text().then((text) => {
                setErrorMessage(`locations backend API error: ${text}`);
                console.log(text);
                return [];
              });
            }
            return response.json();
          })
          .catch((error) => {
            setErrorMessage(getLocationsFetchError(error));
            console.log(error);
            return [];
          });

        callback(response);
      }, 650 /* ms to wait */),
    [setErrorMessage]
  );

  React.useEffect(() => {
    runFetchLocations(extents, (results) => {
      setLocations(results);
      //  console.log(JSON.stringify(results, null, 2));
    });
  }, [extents, runFetchLocations]);

  const router = useRouter();
  React.useEffect(() => {
    console.log(`got center ${JSON.stringify(center, null, 2)} and zoom ${zoom}`);

    if (center?.lat && center?.lng && zoom) {
      router.push(
        {
          pathname: pathUsed + `@${center.lat.toFixed(7)},${center.lng.toFixed(6)},${zoom}z`
        },
        undefined,
        { shallow: true }
      );
    }
  }, [center, zoom]);

  return (
    <>
      <Head>
        <title>{`dir: ${pathUsed}`}</title>
      </Head>
      <Map
        locations={locations}
        initialLocation={initialLocation}
        //  setClick={setClick} // unused but this is how to get click point
        setExtentsForFetch={setExtentsForFetch}
        setZoomForQuery={setZoomForQuery}
        setCenterForQuery={setCenterForQuery}
      />
      <article className="prose m-5" id="dirs">
        {/* multi collection (directory listing) */}
        {data.directories && data.directories.length > 0 && (
          <ul className="list-disc">
            {data.directories.map((directory, index) => (
              <li key={index}>
                <Link href={getDirectoryHref(directory)} legacyBehavior>
                  {directory}
                </Link>
              </li>
            ))}
          </ul>
        )}

        {data.collections && data.collections.length > 0 && (
          <>
            <h1>Locations</h1>
            <ul className="list-disc">
              {data.collections.map((collection) => (
                <li key={collection.id}>
                  <Link
                    href={`/collections/${collection.path}${name_to_path(collection.filename)}`}
                    legacyBehavior
                  >
                    {collection.title}
                  </Link>
                </li>
              ))}
            </ul>
          </>
        )}
      </article>
    </>
  );
}
