import React from 'react';
import Button from '../../components/button';
// todo - user, setErrorMessage,
export default function Home({ setContributingLinks }) {
  React.useEffect(() => {
    setContributingLinks([
      { link: `/frontend/pages/lists/addList.js`, description: `list.js` },
      { link: `/backend/src/queries/list.rs`, description: `backend C/U/D lists` }
    ]);
  }, [setContributingLinks]);

  const [name, setName] = React.useState('');
  const [lat, setLat] = React.useState('');
  const [lon, setLon] = React.useState('');
  const [makePublic, setMakePublic] = React.useState(false);

  const handleRequestLocation = () => {
    navigator.geolocation.getCurrentPosition(function (position) {
      console.log('Latitude is :', position.coords.latitude);
      console.log('Longitude is :', position.coords.longitude);
      setLat(position.coords.latitude);
      setLon(position.coords.longitude);
    });

    return; // todo
  };

  const handleMakePublic = (event) => {
    setMakePublic(event.target.checked);
  };

  const handleSubmit = async () => {
    /* // todo
    const data = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_BASE}/api/list`,
    {
      method: 'POST',
      credentials: 'include',
      body: JSON.stringify({
        location_name: name,
        latitude: lat,
        longitude: lon,
        user_id: user.id,
        location_number: 0, // special case for user collections
        notoriety_score: 0.0, // unused here but set NOT NULL
        ignore_for_nearby_searches: 0, // unused here but set NOT NULL
      })
    })
    .then((response) => {
      if (response.status !== 200) {
        setErrorMessage("can't reach the backend");
        return [];
      }
      return response.json();
    })
    .catch((error) => {
      setErrorMessage(`can't reach backend: ${error.message}`);
      console.log(error);
      return [];
    });
    */
  };

  // todo:
  // support both new list creation and editing
  // submit box only goes active when fields are ready. maybe with hints?
  // list name (text box)
  // location (get user location, or pick on a map, or (todo) zip code)

  // todo - notoriety score is currently not null, switch that to optional
  // initial design - name only

  return (
    <>
      <p>create list</p>
      <label>
        Name:
        <input
          type="text"
          value={name}
          className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
          onChange={(event) => setName(event.target.value)}
        />
      </label>
      <Button
        enabled={true}
        onClick={async () => {
          await handleRequestLocation();
        }}
        className="focus:shadow-outline h-12 w-80 rounded-lg bg-indigo-700 px-6 text-indigo-100 transition-colors duration-150 hover:bg-indigo-800"
        label="get location from my browser"
      />
      <label>
        lat:
        <input
          type="text"
          value={lat}
          className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
          onChange={(event) => setLat(event.target.value)}
        />
      </label>
      <label>
        lon:
        <input
          type="text"
          value={lon}
          className="block w-80 rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
          onChange={(event) => setLon(event.target.value)}
        />
      </label>
      <label>
        <input type="checkbox" defaultChecked={makePublic} onClick={handleMakePublic} />
        make public?
      </label>
      <Button
        enabled={true}
        onClick={async () => {
          await handleSubmit();
        }}
        className="focus:shadow-outline h-12 w-80 rounded-lg bg-indigo-700 px-6 text-indigo-100 transition-colors duration-150 hover:bg-indigo-800"
        label="submit"
      />
    </>
  );
}
