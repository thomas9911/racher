import axios from "axios";
import React, { useEffect } from "react";
import { useParams } from "react-router-dom";
import useSWR from "swr";

import { URL } from "./config";

const fetchGet = (key: string) => (): Promise<any> => {
  return axios({
    baseURL: URL,
    method: "post",
    url: `/get/${key}`,
  }).then(({ data }) => data.data);
};

export const Item = () => {
  let { key } = useParams();
  const { data, error } = useSWR(`/item/${key}`, fetchGet(key));

  if (data === undefined) {
    return <div>LOADINGG.....</div>;
  }

  if (error) {
    return <div>{error.message}</div>;
  }

  return <div>{JSON.stringify(data, null, 2)}</div>;
};
