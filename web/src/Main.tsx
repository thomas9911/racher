import React from "react";
import { Switch, Route } from "react-router-dom";
import { Box } from "grommet";
import { Item } from "./Item";
import { Keys } from "./Keys";
import { JsonForm } from "./JsonForm";

export const Main = () => {
  return (
    <Box
      direction="row"
      border={{ color: "brand", size: "large" }}
      pad="medium"
    >
      <Switch>
        <Route path="/item/:key">
          <Item />
        </Route>
        <Route path="/add">
          <JsonForm />
        </Route>
        <Route path="/">
          <Keys />
        </Route>
      </Switch>
    </Box>
  );
};
