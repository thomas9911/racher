import React from "react";
import { Switch, Route } from "react-router-dom";
import { Box } from "grommet";
import { Item } from "./Get";
import { Keys } from "./Keys";
import { Add } from "./Add";

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
          <Add />
        </Route>
        <Route path="/">
          <Keys />
        </Route>
      </Switch>
    </Box>
  );
};
