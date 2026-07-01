import * as qs from 'qs';
import React from 'react';
import Head from 'next/head';
import { useRouter } from 'next/router';
import { DebounceInput } from 'react-debounce-input';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';
import ItemList from '../components/itemList';
import Button from '../components/buttonLink';
import { getTypesForAutocomplete } from '../components/getTypes';
import { getServerBackendBase } from '../components/backendUrl';

export async function getServerSideProps(context) {
  let queryCleaned = Object.fromEntries(Object.entries(context.query).filter(([, v]) => v != null));

  // if any required thing wasn't specified, redirect after setting defaults
  let redirectNeeded = false;
  if (
    !queryCleaned.searchType ||
    (queryCleaned.patents && queryCleaned.patents != 'true') ||
    !queryCleaned.page ||
    !queryCleaned.perPage ||
    !queryCleaned.orderBy ||
    !queryCleaned.order
  ) {
    redirectNeeded = true;
  }

  // set defaults in the query
  queryCleaned.searchType = queryCleaned.searchType || 'base';
  queryCleaned.patents = queryCleaned.patents == 'true' ? true : undefined;
  queryCleaned.page = queryCleaned.page || '1';
  queryCleaned.perPage = queryCleaned.perPage || '50';
  queryCleaned.orderBy = queryCleaned.orderBy || 'name_then_type';
  queryCleaned.order = queryCleaned.order || 'asc';

  if (redirectNeeded) {
    return {
      redirect: {
        destination: '/search?' + qs.stringify(queryCleaned),
        permanent: false
      }
    };
  }

  const queryString = qs.stringify(queryCleaned);

  let errorMessage = null;
  const backendBase = getServerBackendBase();
  const fetchData = async () => {
    return await fetch(`${backendBase}/api/search?` + queryString)
      .then((response) => {
        if (response.status !== 200) {
          return response.text().then((text) => {
            errorMessage = `backend API error: ${text}`;
            console.log(text);
            return null;
          });
        }
        return response.json();
      })
      .catch((error) => {
        errorMessage = `can't reach backend: ${error.message}`;
        console.log(error);
        return null;
      });
  };

  const data = await fetchData();

  const types = getTypesForAutocomplete(); // todo - move this to getStaticProps() when that's allowed to coexist with getServerSideProps(), see https://github.com/vercel/next.js/discussions/11424

  return {
    props: {
      data,
      queryString,
      types,
      errorMessage
    }
  };
}

const nullIfEmptyQuote = (value) => {
  if (value == '') {
    return null;
  }
  return value;
};

export default function Home({ data, types, errorMessage, setErrorMessage, setContributingLinks }) {
  React.useEffect(() => {
    setContributingLinks([
      { link: `/frontend/pages/search.js`, description: `frontend: search.js` },
      { link: `/backend/src/queries/search.rs`, description: `backend: search.rs` }
    ]);
  }, []);
  setErrorMessage(errorMessage);

  const router = useRouter();
  const query = qs.parse(router.asPath.split(/\?/)[1]);

  const [queryObject, setQueryObject] = React.useState({
    searchType: query.searchType,
    search: query.search,
    name: query.name,
    addToList: query.addToList,
    user: query.user,
    patents: query.patents,
    type: query.type,
    page: query.page,
    perPage: query.perPage,
    orderBy: query.orderBy,
    order: query.order, // asc/desc
    relativeHarvestMin: query.relativeHarvestMin,
    relativeHarvestMax: query.relativeHarvestMax,
    collectionID: query.collectionID,
    collectionPath: query.collectionPath,
    notorietyMin: query.notorietyMin,
    distance: query.distance,
    from: query.from
  });

  const [typeObject, setTypeObject] = React.useState('');

  // create backend query string from the above query params (exclude undefined stuff)
  // store it so we can de-duplicate backend queries
  // update backend stuff on query string change
  // rewrite frontend query string on change

  // set frontend query string for history/bookmarking
  //const [data, setData] = React.useState({});
  React.useEffect(() => {
    // without null and undefined
    let queryCleaned = Object.fromEntries(Object.entries(queryObject).filter(([, v]) => v != null));
    const queryString = qs.stringify(queryCleaned);

    router.query = queryString; // set frontend query string
    router.push(router);

    // update autocomplete if needed - it expects a reference to the array element since we're using an array of objects instead of strings
    console.log(queryObject.type);
    console.dir(types);
    const foundType = types.find((type) => {
      return type.name === queryObject.type;
    });
    console.dir(foundType);

    setTypeObject(foundType); // todo - find the object with .name = this in our types array
  }, [queryObject]);

  const getLinkForPage = (page) => {
    let queryCleaned = Object.fromEntries(Object.entries(queryObject).filter(([, v]) => v != null));
    return 'search?' + qs.stringify({ ...queryCleaned, page }); // todo - is there a cute way to get this page's location and not hard code it?
  };

  const handleOrderByChange = (event) => {
    setQueryObject({ ...queryObject, orderBy: nullIfEmptyQuote(event.target.value), page: '1' });
  };

  const handleOrderChange = (event) => {
    setQueryObject({ ...queryObject, order: nullIfEmptyQuote(event.target.value), page: '1' });
  };

  const handlePerPageChange = (event) => {
    const perPage = nullIfEmptyQuote(event.target.value);
    // only set if if our new perPage is different than previous. also switch to page 1 if we had a change
    if (perPage != queryObject) {
      setQueryObject({ ...queryObject, perPage, page: '1' });
    }
  };

  const handlePatentsChange = (event) => {
    const checked = event.target.checked;
    if (checked) {
      setQueryObject({ ...queryObject, patents: true, page: '1' });
    } else {
      setQueryObject({ ...queryObject, patents: null, page: '1' });
    }
  };

  const handleTypeChange = (type) => {
    setQueryObject({ ...queryObject, type, page: '1' });
  };

  const handleSearchChange = (text) => {
    setQueryObject({ ...queryObject, search: nullIfEmptyQuote(text), page: '1' });
  };

  const handleZipChange = (text) => {
    if (!nullIfEmptyQuote(text)) {
      setQueryObject({ ...queryObject, distance: null, from: null });
    } else {
      setQueryObject({
        ...queryObject,
        distance: '300' /* this is in km - distance chosen to be "nearby" */,
        from: 'zip:' + text,
        page: '1'
      });
    }
  };

  return (
    <>
      <Head>
        <title>{`Search Results`}</title>
      </Head>
      <select
        id="orderBy"
        value={queryObject.orderBy}
        onChange={handleOrderByChange}
        className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
      >
        <option value="name_then_type">name then type</option>
        <option value="type_then_name">type then name</option>
        <option value="patent_expiration">patent expiration</option>
        <option value="harvest_time">harvest time</option>
        <option value="search_quality">search quality</option>
      </select>

      <select
        id="order"
        value={queryObject.order}
        onChange={handleOrderChange}
        className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
      >
        <option value="asc">ascending</option>
        <option value="desc">descending</option>
      </select>

      <select
        id="perPage"
        value={queryObject.perPage}
        onChange={handlePerPageChange}
        className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
      >
        <option value="50">50 per page</option>
        <option value="200">200 per page</option>
        <option value="all">unlimited</option>
      </select>

      <label>
        <input
          type="checkbox"
          defaultChecked={queryObject.patents == true}
          onClick={handlePatentsChange}
        />
        patented only
      </label>

      <Autocomplete
        value={typeObject}
        disablePortal
        id="combo-box-demo"
        options={types}
        getOptionLabel={(option) => option.name}
        sx={{ width: 300 }}
        renderInput={(params) => <TextField {...params} label="Fruit Type" />}
        onChange={(event, option) => {
          handleTypeChange(option?.name);
        }}
      />
      <label>
        search string
        <DebounceInput
          minLength={2}
          debounceTimeout={300}
          onChange={(event) => handleSearchChange(event.target.value)}
          className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
        />
      </label>
      <label>
        zip code
        <DebounceInput
          minLength={2}
          debounceTimeout={300}
          onChange={(event) => handleZipChange(event.target.value)}
          className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
        />
      </label>
      {data?.page && (
        <>
          <h2>{`${data.count} items - page ${data.page}/${data.lastPage ? data.lastPage : ''}`}</h2>
          <Button href={getLinkForPage(1)} enabled={data.page > 1} label="first" />
          <Button
            href={getLinkForPage(parseInt(data.page) - 1)}
            enabled={data.page > 1}
            label="previous"
          />
          {data.patentMidpointPage && (
            <Button href={getLinkForPage(data.patentMidpointPage)} enabled={true} label="current" />
          )}
          <Button
            href={getLinkForPage(parseInt(data.page) + 1)}
            enabled={data.page < parseInt(data.lastPage)}
            label="next"
          />
          <Button
            href={getLinkForPage(parseInt(data.lastPage))}
            enabled={data.page < parseInt(data.lastPage)}
            label="last"
          />
        </>
      )}

      <ItemList data={data}></ItemList>
    </>
  );
}
