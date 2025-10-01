import express, { Request, Response } from "express";

const app = express();
const port = 3000;
app.use(express.json());

app.get("/", (req: Request, res: Response) => {
  res.send("Hello API!");
});
app.post("/echo", (req: Request, res: Response) => {
  res.json({ youSent: req.body });
});

app.listen(port, () => {
  console.log(`🚀 Server is running at http://localhost:${port}`);
});
