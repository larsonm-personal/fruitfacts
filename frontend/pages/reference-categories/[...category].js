import React from 'react';
import Link from 'next/link';
import Head from 'next/head';
import { getServerBackendBase } from '../../components/backendUrl';
import {
  name_to_path,
  path_to_name,
  reference_url
} from '../../components/util';

function collectionPath(collection) {
  return `${collection.path}${collection.filename}`;
}

function collectionHref(collection) {
  return `/collections/${name_to_path(collectionPath(collection))}`;
}

function categoryHref(category) {
  return `/reference-categories/${name_to_path(category)}`;
}

function groupedByRelatedCategory(rows, currentCategory) {
  const groups = {};
  for (const row of rows) {
    for (const category of row.categories || []) {
      if (category != currentCategory) {
        groups[category] = groups[category] || [];
        groups[category].push(row);
      }
    }
  }

  return Object.entries(groups)
    .map(([category, categoryRows]) => ({ category, rows: categoryRows }))
    .sort((left, right) => left.category.localeCompare(right.category));
}

function shouldShowGroups(groups, rows) {
  return (
    groups.length > 1 ||
    groups.some((group) => group.rows.length != rows.length)
  );
}

function ReferenceTable({ rows, currentCategory }) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse text-left text-sm">
        <thead>
          <tr className="border-b border-gray-300">
            <th className="py-2 pr-4">Reference</th>
            <th className="py-2 pr-4">Published</th>
            <th className="py-2 pr-4">Categories</th>
            <th className="py-2 pr-4">Source</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const collection = row.collection;
            const sourceUrl = reference_url(collection.url);
            return (
              <tr key={collection.id} className="border-b border-gray-200">
                <td className="py-2 pr-4 align-top">
                  <Link href={collectionHref(collection)}>
                    {collection.title || collection.filename}
                  </Link>
                  {collection.needs_help == 1 && (
                    <span className="ml-2 text-xs text-gray-700">
                      needs help
                    </span>
                  )}
                </td>
                <td className="py-2 pr-4 align-top">{collection.published}</td>
                <td className="py-2 pr-4 align-top">
                  <ul className="flex flex-wrap gap-x-3 gap-y-1">
                    {(row.categories || []).map((category) => (
                      <li key={category}>
                        {category == currentCategory ? (
                          <span>{category}</span>
                        ) : (
                          <Link href={categoryHref(category)}>{category}</Link>
                        )}
                      </li>
                    ))}
                  </ul>
                </td>
                <td className="py-2 pr-4 align-top">
                  {sourceUrl && <a href={sourceUrl}>source</a>}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export async function getServerSideProps(context) {
  let errorMessage = null;
  const categoryPath = context.query.category.join('/');
  const categoryName = path_to_name(categoryPath);
  const backendBase = getServerBackendBase();
  const data = await fetch(
    `${backendBase}/api/reference_categories/${name_to_path(categoryName)}`
  )
    .then(async (response) => {
      if (response.status !== 200) {
        const text = await response.text();
        errorMessage = `backend API error: ${text}`;
        console.log(response.status + ': ' + text);
        return { category: categoryName, collections: [] };
      }
      return response.json();
    })
    .catch((error) => {
      errorMessage = `can't reach backend: ${error.message}`;
      console.log(error);
      return { category: categoryName, collections: [] };
    });

  return { props: { data, errorMessage } };
}

export default function ReferenceCategory({
  data,
  errorMessage,
  setErrorMessage,
  setContributingLinks
}) {
  React.useEffect(() => {
    setContributingLinks([
      {
        link: `/frontend/pages/reference-categories/[...category].js`,
        description: `reference category page`
      }
    ]);
  }, []);

  React.useEffect(() => {
    setErrorMessage(errorMessage);
  }, [errorMessage, setErrorMessage]);

  const rows = data.collections || [];
  const groups = groupedByRelatedCategory(rows, data.category);
  const showGroups = shouldShowGroups(groups, rows);

  return (
    <article className="m-5 max-w-6xl">
      <Head>
        <title>{`Reference category: ${data.category}`}</title>
      </Head>
      <header className="mb-6">
        <h1 className="text-3xl font-semibold">{data.category}</h1>
        <div className="mt-2 text-sm text-gray-700">
          {rows.length} references
        </div>
      </header>

      {showGroups ? (
        groups.map((group) => (
          <section key={group.category} className="mb-8">
            <h2 className="mb-2 text-xl font-semibold">
              <Link href={categoryHref(group.category)}>{group.category}</Link>
            </h2>
            <ReferenceTable rows={group.rows} currentCategory={data.category} />
          </section>
        ))
      ) : (
        <ReferenceTable rows={rows} currentCategory={data.category} />
      )}
    </article>
  );
}
