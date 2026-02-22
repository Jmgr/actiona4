≫ await Directory.listEntries("/home")
[ DirectoryEntry(path: /home/jmgr, fileName: jmgr, isFile: false, isDirectory: true, isSymlink: false, size: 4096) ]
≫ await Directory.listEntries("/home/jmgr")
error: Error: IO Error: No such file or directory (os error 2)
