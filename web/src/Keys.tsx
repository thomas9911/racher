import axios from "axios";
import React, { useEffect } from "react";
import useSWR from "swr";
import { List, TextInput, Spinner } from "grommet";
import { URL } from "./config";
import { AnchorLink } from "./AnchorLink";

const fetchKeys = (): Promise<string[]> => {
  return axios({
    baseURL: URL,
    method: "post",
    url: "/keys",
  }).then(({ data }) => data.keys);
};

const keyItems = (data: string[]): { key: JSX.Element }[] => {
  return data.map((key) => ({
    key: <AnchorLink label={key} to={`/item/${key}`} />,
  }));
};

export const Keys = () => {
  const { data, error } = useSWR("/keys", fetchKeys);
  const [filterValue, setFilterValue] = React.useState("");
  const [filteredData, setFilteredData] = React.useState([]);

  useEffect(() => {
    if (data === undefined) {
      return;
    }
    data.sort();

    if (filterValue) {
      setFilteredData(keyItems(data.filter((x) => x.includes(filterValue))));
    } else {
      setFilteredData(keyItems(data));
    }
  }, [data, filterValue]);

  if (data === undefined) {
    return <Spinner />;
  }

  if (error) {
    return <div>{error.message}</div>;
  }

  return (
    <div>
      <TextInput
        placeholder="type here"
        value={filterValue}
        type="text"
        onChange={(event) => setFilterValue(event.target.value)}
      />
      <List step={200} primaryKey="key" data={filteredData} />;
    </div>
  );
};
