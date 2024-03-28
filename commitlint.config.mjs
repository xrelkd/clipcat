const Configuration = {
  extends: [
    process.env.IN_NIX_SHELL ? process.env.COMMITLINT_PRESET : "@commitlint/config-conventional",
  ],
  rules: {
    "body-max-line-length": [2, "always", 150],
    "header-max-length": [2, "always", 150],
  },
};

export default Configuration;
