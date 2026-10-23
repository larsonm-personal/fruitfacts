import Image from 'next/image';
import React from 'react';
import Link from 'next/link';

export async function getStaticProps() {
  var fs = require('fs');
  const path = require('path');

  const icons_dir = path.join(process.cwd(), 'public', 'fruit_icons');
  return {
    props: {
      icons: fs.readdirSync(icons_dir)
    }
  };
}

// next.js advises keeping logic client-side in 404s so we can limit server load. ok?
export default function Custom404({ icons, setErrorMessage, setContributingLinks }) {
  React.useEffect(() => {
    setContributingLinks([
      { link: `/frontend/pages/404.js`, description: `404.js` },
      { link: `/plant_database/facts.json5`, description: `fact list` }
    ]);
  }, [setContributingLinks]);

  const [icon, setIcon] = React.useState();
  const [fact, setFact] = React.useState();
  React.useEffect(() => {
    const fetchData = async () => {
      const fact = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_BASE}/api/fact`)
        .then((response) => {
          if (response.status !== 200) {
            setErrorMessage("can't reach the backend to fetch a fact");
            return;
          }
          return response.json();
        })
        .catch((error) => {
          setErrorMessage("can't reach the backend to fetch a fact");
          console.log(error);
          return;
        });

      setFact(fact);
      setIcon(icons[Math.floor(Math.random() * icons.length)]);
    };

    fetchData();
  }, [icons, setErrorMessage]);

  return (
    <div className="grid place-items-center">
      <article className="prose m-5">
        <div className="grid place-items-center">
          <p>
            <b>Fact:</b> This is a 404 page
          </p>
          {fact?.fact && (
            <p>
              <b>Fact:</b> {`${fact.fact} `}
              <a href={` ${fact.reference}`}>[ref]</a>
            </p>
          )}
          {icon && (
            <Link
              href={`search?searchType=base&type=${icon.substr(
                0,
                icon.indexOf('.')
              )}&page=1&perPage=50&orderBy=name_then_type&order=asc`}
            >
              <Image
                width={192}
                height={192}
                alt={icon.replace(/\.svg$/, '')}
                className="h-48 w-48 object-contain"
                src={'/fruit_icons/' + icon}
              />
            </Link>
          )}
        </div>
      </article>
    </div>
  );
}
