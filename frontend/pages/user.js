import React from 'react';

export default function Home({ setContributingLinks, setErrorMessage }) {
  React.useEffect(() => {
    setContributingLinks([{ link: `/frontend/pages/user.js`, description: `user.js` }]);
  }, [setContributingLinks]);

  const [fullUser, setFullUser] = React.useState(null);

  React.useEffect(() => {
    // todo - this is too many lines for what it does. simplify (and other occurrences)
    const fetchData = async () => {
      const data = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_BASE}/api/getFullUser`, {
        credentials: 'include'
      })
        .then((response) => {
          if (response.status === 200) {
            return response.json();
          } else {
            setErrorMessage("couldn't get full user");
            return null;
          }
        })
        .catch((error) => {
          setErrorMessage(`couldn't get full user: ${error.message}`);
          console.log(error);
          return null;
        });

      setFullUser(data);
    };

    fetchData();
  }, [setErrorMessage]);

  return <>{fullUser && <p>{JSON.stringify(fullUser)}</p>}</>;
}
