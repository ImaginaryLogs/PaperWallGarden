while inotifywait -r -e modify,create,delete /mnt/e/The\ Studio/\[06\]\ Asynchronous\ Files/QuezonRepository/QuezonRepository/; do
    rsync -avz --delete /mnt/e/The\ Studio/\[06\]\ Asynchronous\ Files/QuezonRepository/QuezonRepository/ ~/PaperWallGarden/content/ --exclude="*.pdf"
done