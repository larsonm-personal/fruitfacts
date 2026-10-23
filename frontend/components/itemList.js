import React from 'react';
import Link from 'next/link';
import { formatHarvestTime, formatPatentDate } from './functions';
import { name_to_path } from './util';

export default function Home({ data }) {
  return (
    <>
      <article className="prose m-5">
        <ul className="list-none">
          {data?.basePlants && (
            <>
              {data.basePlants.map((item, index) => (
                <>
                  <li key={index}>
                    <img
                      className="my-0 mx-2 inline h-6 w-6 object-contain"
                      src={'/fruit_icons/' + item.type + '.svg'}
                    />
                    <Link
                      href={`/plant/${name_to_path(item.type + '/' + item.name)}`}

                    >
                      {item.name + ' ' + item.type}
                    </Link>
                    {item.marketing_name && <> (marketed under the {item.marketing_name} brand)</>}{' '}
                    {data.query.orderBy == 'harvest_time' && (
                      <>{formatHarvestTime(item.harvest_relative)}</>
                    )}
                    {(data.query.patents || data.query.orderBy == 'patent_expiration') && (
                      <>
                        {' '}
                        ({formatPatentDate(item.uspp_expiration, item.uspp_expiration_estimated)})
                      </>
                    )}
                  </li>
                </>
              ))}
            </>
          )}
        </ul>
      </article>
    </>
  );
}
