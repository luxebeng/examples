import { useEffect, useMemo, useState } from "react";
import Dialog from "../Dialog";
import StorageDialogPlugins from "./pages/StorageDialogPlugins";
import StorageDialogFiles from "./pages/StorageDialogFiles";

interface StorageDialogProps {
    title?: string;
    filter: StorageFilter;
    show?: boolean;
    filename?: string;
    onOk?: (filename: string) => void;
    okText?: string | React.ReactNode;
    onCancel?: () => void;
}

interface IconMemeType {
    [key: string]: string;
    "wasm": string;
    "json": string;
    "octet-stream": string;
    "directory": string;
}

const iconMemeType: IconMemeType = {
    "wasm": "https://upload.wikimedia.org/wikipedia/commons/thumb/1/1f/WebAssembly_Logo.svg/900px-WebAssembly_Logo.svg.png?20171120175633",
    "json": "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c9/JSON_vector_logo.svg/1200px-JSON_vector_logo.svg.png",
    "octet-stream": "https://upload.wikimedia.org/wikipedia/commons/thumb/1/1f/WebAssembly_Logo.svg/900px-WebAssembly_Logo.svg.png?20171120175633",
    "directory": "https://cdn4.iconfinder.com/data/icons/48-bubbles/48/15.Folder-512.png"
};

const StorageDialog = ({ title, filter, show, filename, onOk, okText, onCancel }: StorageDialogProps) => {
    const [page, setPage] = useState("plugins");
    const [pluginId, setPluginId] = useState(-1);
    const [file, setFile] = useState<StorageFile | null>(null);

    useEffect(() => {
        if (pluginId !== -1) {
            setPage("files");
        }
    }, [pluginId]);

    useEffect(() => {
        if (file) {
            onOk?.(file.name);
        }
    }, [file]);

    const handleDialogFilesClose = () => {
        setPage("plugins");
        setPluginId(-1);
    };

    return <>
        <Dialog>
            <Dialog.Header title={title} />
            <Dialog.Body>
                {page === "plugins" && <StorageDialogPlugins onPluginSelect={setPluginId}  />}
                {page === "files" && <StorageDialogFiles filter={filter} plugin={pluginId} onClose={handleDialogFilesClose} onFileSelect={setFile} />}
            </Dialog.Body>
            <Dialog.Footer>
                <div className="panel">
                    <button className="btn" onClick={onCancel}>Close<i className="fas fa-ban" /></button>
                </div>
            </Dialog.Footer>
        </Dialog>
    </>;
};

export default StorageDialog;