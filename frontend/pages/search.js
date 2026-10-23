import * as qs from 'qs';
import React from 'react';
import Head from 'next/head';
import { useRouter } from 'next/router';
import DebounceInput from '../components/debounceInput';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';
import ItemList from '../components/itemList';
import Button from '../components/buttonLink';
import { getTypesForAutocomplete } from '../components/getTypes';

export async function getServerSideProps(context) {
  let queryCleaned = Object.fromEntries(Object.entries(context.query).filter(([, v]) => v != null));
  const queryString = qs.stringify(queryCleaned);

  let errorMessage = null;
  const fetchData = async () => {
    return await fetch(`${process.env.NEXT_PUBLIC_BACKEND_BASE}/api/search?` + queryString)
      .then((response) => {
        if (response.status !== 200) {
          errorMessage = `backend API error ${response.status}`;
          return null;
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
  }, [setContributingLinks]);
  React.useEffect(() => setErrorMessage(errorMessage), [errorMessage, setErrorMessage]);

  const router = useRouter();
  const query = qs.parse(router.asPath.split(/\?/)[1]);

  // get defaults from query - see struct in search.rs
  let querySearchType = query.searchType || 'base';
  let querySearch = query.search || undefined;
  let queryName = query.name || undefined;
  let queryPatents = query.patents == 'true' ? true : null;
  let queryType = query.type || undefined; // apple, pear, etc.
  let queryPage = query.page || '1';
  let queryPerPage = query.perPage || '50';
  let queryOrderBy = query.orderBy || 'name_then_type';
  let queryOrder = query.order || 'asc';
  let queryRelativeHarvestMin = query.relativeHarvestMin || undefined;
  let queryRelativeHarvestMax = query.relativeHarvestMax || undefined;

  let queryCollectionID = query.collectionID || undefined;
  let queryCollectionPath = query.collectionPath || undefined;

  let queryNotorietyMin = query.notorietyMin || undefined;

  let queryDistance = query.distance || undefined; // todo
  let queryFrom = query.from || undefined; // todo

  const [queryObject, setQueryObject] = React.useState({
    searchType: querySearchType,
    search: querySearch,
    name: queryName,
    patents: queryPatents,
    type: queryType,
    page: queryPage,
    perPage: queryPerPage,
    orderBy: queryOrderBy,
    order: queryOrder, // asc/desc
    relativeHarvestMin: queryRelativeHarvestMin,
    relativeHarvestMax: queryRelativeHarvestMax,
    collectionID: queryCollectionID,
    collectionPath: queryCollectionPath,
    notorietyMin: queryNotorietyMin,
    distance: queryDistance,
    from: queryFrom
  });

  const typeObject = types.find((type) => type.name === queryObject.type) || null;

  React.useEffect(() => {
    const query = Object.fromEntries(Object.entries(queryObject).filter(([, v]) => v != null));
    const queryString = qs.stringify(query);
    if (router.asPath.split('?')[1] !== queryString) {
      router.push({ pathname: router.pathname, query }, undefined, { scroll: false });
    }
  }, [queryObject, router]);

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
