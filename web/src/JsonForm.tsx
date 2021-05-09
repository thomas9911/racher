import axios from "axios";
import { Form, FormField, TextInput, Box, Button, TextArea } from "grommet";
import React, { useEffect, useState } from "react";
import { URL } from "./config";
import ReactJson from "react-json-view";
import Modal from "react-modal";
import { customTheme } from "./theme";

const DEFAULT_FORM_STATE = { key: "", data: "" };

interface AddProps {
  itemKey?: string;
  data?: string;
}

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

const validateData = ({ data }, errors = {}) => {
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

const validate = ({ key, data }) => {
  let errors = {};
  if (!key) {
    errors = { ...errors, key: "key should not be empty" };
  }

  return validateData({ data }, errors);
};

Modal.setAppElement("#root");

const JsonModal = ({
  modalIsOpen,
  afterOpenModal,
  closeModal,
  initValue,
  setParentValue,
}) => {
  const [value, setValue] = useState({ data: initValue });
  useEffect(() => {
    setValue({ data: initValue });
  }, [initValue]);

  const [errors, setErrors] = React.useState({});

  const onSubmit = ({ value }) => {
    const validateErrors = validateData(value);
    if (objectEmpty(validateErrors)) {
      return setErrors(validateErrors);
    }
    setParentValue(value);
  };

  return (
    <Modal
      isOpen={modalIsOpen}
      onAfterOpen={afterOpenModal}
      onRequestClose={closeModal}
      style={{
        content: {
          color: customTheme.global.colors["accent-1"],
          background: customTheme.global.colors.background,
        },
      }}
    >
      <Form
        value={value}
        onChange={({ data }) => setValue(() => ({ data: data }))}
        onReset={() => setValue(({}) => ({ data: "" }))}
        onSubmit={onSubmit}
        errors={errors}
      >
        <FormField
          name="data"
          htmlFor="data-input-id"
          label="Data (should be JSON format)"
        >
          <TextArea id="data-input-id" name="data" fill rows={25} />
        </FormField>

        <Box direction="row" gap="medium" margin="medium">
          <Button type="submit" primary label="Submit" />
          <Button type="reset" label="Reset" />
          <Button onClick={closeModal} label="Close" />
        </Box>
      </Form>
    </Modal>
  );
};

export const JsonForm = ({ itemKey = "", data = "" }: AddProps) => {
  const formState = { key: itemKey, data };
  const [value, setValue] = React.useState(formState);
  const [errors, setErrors] = React.useState({});
  const [success, setSuccess] = React.useState(false);
  const [objectData, setObjectData] = React.useState({});
  const [modalIsOpen, setIsOpen] = React.useState(false);
  const [modalValue, setModalValue] = React.useState("");

  React.useEffect(() => {
    try {
      setObjectData(JSON.parse(value.data));
    } catch (error) {}
  }, [value]);

  React.useEffect(() => {
    setModalValue(value.data);
  }, [value]);

  const updater = ({ updated_src }) => {
    setValue((prev) => ({
      ...prev,
      ...{ data: JSON.stringify(updated_src) },
    }));
  };

  const openModal = () => {
    setModalValue(value.data);
    setIsOpen(true);
  };

  const afterOpenModal = () => {
    // references are now sync'd and can be accessed.
    // subtitle.style.color = '#f00';
  };

  const closeModal = () => {
    setIsOpen(false);
  };

  const onSubmit = ({ value }) => {
    const validateErrors = validate(value);
    if (objectEmpty(validateErrors)) {
      return setErrors(validateErrors);
    }
    setData(value).then((msg) => setSuccess(msg === "ok"));
  };

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
        onSubmit={onSubmit}
        errors={errors}
      >
        <FormField name="key" htmlFor="text-input-id" label="Name">
          <TextInput id="text-input-id" name="key" />
        </FormField>
        {/* <FormField
          name="data"
          htmlFor="data-input-id"
          label="Data (should be JSON format)"
        >
          <TextArea id="data-input-id" name="data" fill />
          
        </FormField> */}

        <Box direction="column" gap="medium" margin="medium">
          <Button onClick={openModal} label="Customize JSON text" />
          <ReactJson
            name={false}
            src={objectData}
            iconStyle="triangle"
            theme="monokai"
            onEdit={updater}
            onAdd={updater}
            onDelete={updater}
          />
        </Box>

        <Box direction="row" gap="medium" margin="medium">
          <Button type="submit" primary label="Submit" />
          <Button type="reset" label="Reset" />
        </Box>
      </Form>
      <JsonModal
        modalIsOpen={modalIsOpen}
        afterOpenModal={afterOpenModal}
        closeModal={closeModal}
        initValue={modalValue}
        setParentValue={(updated) =>
          setValue((prev) => ({
            ...prev,
            ...updated,
          }))
        }
      />
    </div>
  );
};
