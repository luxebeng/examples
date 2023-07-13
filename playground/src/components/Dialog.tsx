interface DialogProps {
    show?: boolean;
    title?: string;
    children?: React.ReactNode;
    footer?: React.ReactNode;
}

const Header = ({title, children}: {title?: string, children?: React.ReactNode}) => {
    return <div className="dialog-header">
        <div className="dialog-title">{title}</div>
        {children}
    </div>;
}

const Body = ({children}: {children?: React.ReactNode}) => {
    return <div className="dialog-body">
        {children}
    </div>;
}

const Footer = ({children}: {children?: React.ReactNode}) => {
    return <div className="dialog-footer">
        {children}
    </div>;
}

const Dialog = ({show, title, children, footer}: DialogProps) => {
    return <>
        <div className="drop-shadow">
            <div className="dialog">
                {title && <Header title={title} />}
                {children}
                {footer && <Footer>{footer}</Footer>}
            </div>
        </div>
    </>;
};

Dialog.Header = Header;
Dialog.Body = Body;
Dialog.Footer = Footer;

export default Dialog;