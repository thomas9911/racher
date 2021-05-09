import React from "react";
import { Link, LinkProps } from "react-router-dom";
import { Anchor } from "grommet";
import { AnchorProps } from "grommet/components/Anchor";

type AnchorLinkProps = LinkProps &
  AnchorProps &
  Omit<JSX.IntrinsicElements["a"], "color">;

export const AnchorLink: React.FC<AnchorLinkProps> = (props): JSX.Element => {
  return <Anchor as={Link} {...props} />;
};
