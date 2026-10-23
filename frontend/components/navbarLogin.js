import React from 'react';
import Link from 'next/link';

export default function Home({ user, setUser }) {
  React.useEffect(() => {
    const fetchData = async () => {
      const data = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_BASE}/api/checkLogin`, {
        credentials: 'include'
      })
        .then((response) => {
          if (response.status !== 200) {
            return {};
          } else {
            return response.json();
          }
        })
        .catch((error) => {
          console.log(error);
          return {};
        });

      setUser(data);
    };

    fetchData();
  }, [setUser]);

  function logOut() {
    fetch(`${process.env.NEXT_PUBLIC_BACKEND_BASE}/api/logout`, {
      method: 'POST',
      credentials: 'include'
    })
      .then((response) => {
        if (response.status === 200) {
          setUser(null);
        }
        return response.json();
      })
      .catch((error) => {
        console.log(error);
      });
  }

  return (
    <>
      {user?.name ? (
        <p>
          logged in as <Link href="/user/">{user.name}</Link>
          <button
            className="mt-4 mr-4 ml-1 block text-teal-200 hover:text-white lg:mt-0 lg:inline-block"
            onClick={logOut}
          >
            log out
          </button>
        </p>
      ) : (
        <Link href="/login">log in</Link>
      )}
    </>
  );
}
