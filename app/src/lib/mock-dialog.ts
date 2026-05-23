export async function open(options?: {
  multiple?: boolean;
  directory?: boolean;
  filters?: { name: string; extensions: string[] }[];
}): Promise<string | string[] | null> {
  if (options?.directory === true) return '/tmp/mock-backup-dir';
  if (options?.multiple === true) return ['/tmp/test-file.txt'];
  return '/tmp/test-file.txt';
}

