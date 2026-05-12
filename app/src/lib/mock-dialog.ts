export async function open(options?: {
  multiple?: boolean;
  filters?: { name: string; extensions: string[] }[];
}): Promise<string | string[] | null> {
  if (options?.multiple === true) return ['/tmp/test-file.txt'];
  return '/tmp/test-file.txt';
}

export async function save(_options?: {
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}): Promise<string | null> {
  return '/tmp/output-file';
}
