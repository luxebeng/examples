import { createTheme } from '@uiw/codemirror-themes';
import CodeMirror from '@uiw/react-codemirror';
import { tags as t } from '@lezer/highlight';

type ZkmoveInputsProps = {
  value: string;
  onChange: (value: string) => void;
};

const codeTheme = createTheme({
  theme: 'dark',
  settings: {
    background: '#19181F',
    foreground: '#ffffff',
    caret: '#ffffff',
    selection: '#036dd626',
    selectionMatch: '#036dd626',
    lineHighlight: '#19181F',
    gutterBackground: '#19181F',
    gutterForeground: '#ffffff'
  },
  styles: [{ tag: t.comment, color: '#787b8099' }]
});

const ZkmoveInputs = (props: ZkmoveInputsProps): JSX.Element => (
  <div className="flex w-full overflow-auto">
    <CodeMirror
      value={props.value}
      theme={codeTheme}
      onChange={props.onChange}
      basicSetup={{
        foldGutter: true,
        highlightActiveLineGutter: true,
        dropCursor: true,
        allowMultipleSelections: false,
        indentOnInput: false,
        lineNumbers: true,
        syntaxHighlighting: true,
        bracketMatching: true,
        autocompletion: true,
        highlightActiveLine: true
      }}
      className="grow overflow-auto pr-3"
    />
  </div>
);

export default ZkmoveInputs;
