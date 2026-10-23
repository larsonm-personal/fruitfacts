import Link from 'next/link';
import { repositoryUrl, sourceUrl } from './sourceLinks';
import Search from './navbarSearch';
import Login from './navbarLogin';
import Tooltip from '@mui/material/Tooltip';
import React from 'react';

let links = [
  { name: 'locations', href: '/dirs/' },
  { name: 'plants', href: '/plants' },
  {
    name: 'US patents',
    href: '/search?searchType=base&patents=true&page=1&perPage=50&orderBy=patent_expiration'
  }
];

export default function Home({ user, setUser, contributingLinks }) {
  const [open, setOpen] = React.useState(false);
  const handleEditClick = () => {
    setOpen(!open);
  };

  return (
    <nav className="flex flex-wrap items-center justify-between bg-teal-500 p-6">
      <div className="mr-6 flex shrink-0 items-center text-white">
        <Link href="/" className="text-xl font-semibold tracking-tight">
          fruitfacts
        </Link>
      </div>
      <div className="mr-6 flex shrink-0 items-center text-white">
        <Search />
      </div>
      <div className="block w-full grow lg:flex lg:w-auto lg:items-center">
        <div className="text-sm lg:grow">
          {links.map((link) => (
            <Link
              key={link.name}
              href={link.href}
              className="mt-4 mr-4 block text-teal-200 hover:text-white lg:mt-0 lg:inline-block"
            >
              {link.name}
            </Link>
          ))}
          <>
            {user?.name && (
              <Link
                key="my lists"
                href={`/lists/lists?user_id=${user.id}`}
                className="mt-4 mr-4 block text-teal-200 hover:text-white lg:mt-0 lg:inline-block"
              >
                my lists
              </Link>
            )}
          </>
        </div>
        <div className="mr-6">
          <Tooltip
            open={open}
            title={
              <React.Fragment>
                <a href={repositoryUrl}>fruitfacts on github</a>
                <ul className="list-disc">
                  {contributingLinks && (
                    <>
                      {contributingLinks.map((link, index) => (
                        <li key={index}>
                          <a href={sourceUrl(link.link)}>{link.description}</a>
                        </li>
                      ))}
                    </>
                  )}
                </ul>
              </React.Fragment>
            }
          >
            <button onClick={handleEditClick}>edit this page</button>
          </Tooltip>
        </div>
        <div>
          <Login user={user} setUser={setUser} />
        </div>
      </div>
    </nav>
  );
}
