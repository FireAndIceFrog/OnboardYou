import { RemoteModule, RemotePackageConfig } from "./domain/types";

const remotePackages: RemotePackageConfig[] = [
  {
    package: "configApp",
    entry: () => import('configApp/App')  as unknown as Promise<RemoteModule>,
    path: "config/*",
    useGlobal: true,
    version: 1
  }
]

export default remotePackages;