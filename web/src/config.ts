let URL = "/";
if (process.env.NODE_ENV === 'development') {
    URL = "http://localhost:9226";
}

export {URL};
