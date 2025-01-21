const iconcaptcha = require("./index.node");
const fs = require("fs");
const dir = "./captchas";
const fileList = fs.readdirSync(dir);
fileList.forEach((file) => {
  const filePath = `${dir}/${file}`;
  try {
    const fileContent = fs.readFileSync(filePath);
    const buffer = new Buffer.from(fileContent);
    const bs64 = buffer.toString("base64");
    console.log(iconcaptcha.solve(bs64));
  } catch (e) {
    console.error(e);
  }
});
