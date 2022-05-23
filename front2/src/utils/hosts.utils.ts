export function getColor(state: string) {
  switch (state) {
    case "waiting":
    case "active":
      return "blue";
    case "failed":
      return "red";
    case "completed":
      return "green";
    case "delayed":
      return "yellow";
    default:
      return "grey";
  }
}
