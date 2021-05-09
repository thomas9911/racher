import axios from "axios";
import { Form, FormField, TextInput, Box, Button, TextArea } from "grommet";
import React from "react";
import { URL } from "./config";

const DEFAULT_FORM_STATE = { key: "", data: "" };

const objectEmpty = (obj: object): boolean => {
  return Object.keys(obj).length !== 0;
};

const setData = ({ key, data }) => {
  return axios({
    baseURL: URL,
    url: `/set/${key}`,
    data,
    headers: { "content-type": "application/json" },
    method: "post",
  }).then(({ data }) => data.status);
};

const validate = ({ key, data }) => {
  let errors = {};
  if (!key) {
    errors = { ...errors, key: "key should not be empty" };
  }
  if (!data) {
    errors = { ...errors, data: "data should not be empty" };
  }
  if (objectEmpty(errors)) {
    return errors;
  }
  try {
    JSON.parse(data);
    return errors;
  } catch (error) {
    return { ...errors, data: "data should be json" };
  }
};

export const Add = () => {
  const [value, setValue] = React.useState(DEFAULT_FORM_STATE);
  const [errors, setErrors] = React.useState({});
  const [success, setSuccess] = React.useState(false);

  return (
    <div>
      {success ? (
        <Box background="status-ok" direction="row">
          winner
        </Box>
      ) : (
        <div></div>
      )}
      <Form
        value={value}
        onChange={(nextValue) => setValue(nextValue)}
        onReset={() => setValue(DEFAULT_FORM_STATE)}
        onSubmit={({ value }) => {
          const validateErrors = validate(value);
          if (objectEmpty(validateErrors)) {
            return setErrors(validateErrors);
          }
          setData(value).then((msg) => setSuccess(msg === "ok"));
        }}
        errors={errors}
      >
        <FormField name="key" htmlFor="text-input-id" label="Name">
          <TextInput id="text-input-id" name="key" />
        </FormField>
        <FormField name="data" htmlFor="data-input-id" label="Data">
          <TextArea id="data-input-id" name="data" />
        </FormField>
        <Box direction="row" gap="medium">
          <Button type="submit" primary label="Submit" />
          <Button type="reset" label="Reset" />
        </Box>
      </Form>
    </div>
  );
};
