# MediaNamer Project Idea
A Linux movie and tv show episode file renamer.

## Main Function
- Must have a GUI
- Have the ability for the user to drag a file or multiple files to the GUI and show of list of files to potentially be renamed.
- The user can select from a list of media information sources, a list for movies and a list for TV episodes, to use provide the information to rename the file.  
- The user can initiate a pattern match from the initial filename and any useful media information properties of the file to determine the proper movie or TV episode name information.
- Matched movie or tv episode information is shown to the user in the new filename format defined by the user.
- If the user approves the new filenames, the application will rename the files to the new file name and remove the movie from the list displayed.
- Movie information sources include: The Movie Database (TMDB) (themoviedb.org), IMDB (imdb.com)
- TV episode information sources include: The Movie Database (TMDB) (themoviedb.org), IMDB (imdb.com), TheTVDB (thetvdb.com), TVmaze (tvmaze.com)
- Have a file format format naming convention similar to Filebot, Sonarr, and Radarr with an instruction window documenting the naming conventions while the user is developing their custom file name format.
- Have the ability to obtain media information from the media file for us in the file naming process, such as but not limited to Title, Video Resolution, and Video Codec.  Use of MediaInfo is an option.

# Tools
- Written in Rust.
- Provide an install packages for Debian, Arch, and Redhat based distributions.
- Write unit tests
