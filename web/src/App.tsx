import React from "react";
import { BrowserRouter as Router } from "react-router-dom";
import ReactDOM from "react-dom";
import { Grommet, Nav } from "grommet";
import { Grid, Add } from "grommet-icons/icons";
import { AnchorLink } from "./AnchorLink";
import { Main } from "./Main";
import { customTheme } from "./theme";

const App = () => {
  return (
    <Router basename="/dashboard">
      <Grommet theme={customTheme}>
        <Nav direction="row" background="brand" pad="medium">
          <AnchorLink label={<Grid />} to="/" hoverIndicator />
          <AnchorLink label={<Add />} to="/add" hoverIndicator />
        </Nav>
        <Main />
      </Grommet>
    </Router>
  );
};

const rootElement = document.getElementById("root");
if (rootElement.hasChildNodes()) {
  ReactDOM.hydrate(<App />, rootElement);
} else {
  ReactDOM.render(<App />, rootElement);
}
