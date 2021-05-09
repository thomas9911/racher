import axios from "axios";
import { Spinner } from "grommet";
import React from "react";
import { useParams } from "react-router-dom";
import useSWR from "swr";

import { JsonForm } from "./JsonForm";
import { URL } from "./config";

const fetchItem = (key: string) => (): Promise<any> => {
  return axios({
    baseURL: URL,
    method: "post",
    url: `/get/${key}`,
  }).then(({ data }) => data.data);
};

export const Item = () => {
  let { key } = useParams();
  const { data, error } = useSWR(`/item/${key}`, fetchItem(key));

  if (data === undefined) {
    return <Spinner />;
  }

  if (error) {
    return <div>{error.message}</div>;
  }

  // return <div>{JSON.stringify(data, null, 2)}</div>;
  return <JsonForm itemKey={key} data={JSON.stringify(data, null, 2)} />;
};
