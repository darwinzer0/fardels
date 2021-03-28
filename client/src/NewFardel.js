import React, { Component } from 'react';
import { connect } from 'react-redux';
import Resizer from 'react-image-file-resizer';
import Autolinker from 'autolinker';

//Material-UI
import Box from '@material-ui/core/Box';
import Button from '@material-ui/core/Button';
import Paper from '@material-ui/core/Paper';
import PublishIcon from '@material-ui/icons/Publish';
import TextField from '@material-ui/core/TextField';
import Divider from '@material-ui/core/Divider';
import Grid from '@material-ui/core/Grid';
import IconButton from '@material-ui/core/IconButton';
import PhotoCamera from '@material-ui/icons/PhotoCamera';
import Typography from '@material-ui/core/Typography';
import InputAdornment from '@material-ui/core/InputAdornment';
import BackupTwoToneIcon from '@material-ui/icons/BackupTwoTone';
import { withStyles } from '@material-ui/core/styles';

import { executeCarryFardel,
         setNewFardelMessage,
         setNewFardelContents,
         setNewFardelThumbnail,
         clearNewFardel, } from './store/Actions';

const useStyles = theme => ({
  button: {
    margin: theme.spacing(1),
  },
  buttonBox: {
    width: '100%',
  },
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
  textFieldContents: {
    margin: theme.spacing(1),
  },
  textFieldPrice: {
    margin: theme.spacing(1),
    width: '25ch',
  },
  divider: {
    height: '1px',
    marginLeft: '20px',
    marginRight: '20px',
  },
  heading: {
    fontSize: theme.typography.pxToRem(15),
    fontWeight: theme.typography.fontWeightRegular,
  },
  paper: {
    margin: 8,
    padding: theme.spacing(3),
  },
});

class NewFardel extends Component {

  constructor(props) {
    super(props);
    this.thumbnailChangedHandler = this.thumbnailChangedHandler.bind(this);
    this.state = {
      newImage: ''
    }
  }

  newFardelMessageChangedHandler(event) {
    const { newFardelMessage, setNewFardelMessage } = this.props;

    if (((new TextEncoder().encode(event.target.value))).length <= 280) {
      setNewFardelMessage(event.target.value);
    } else {
      setNewFardelMessage(newFardelMessage);
    }
  }

  newFardelContentsTextInputHandler(event) {
    const { setNewFardelContents, newFardelContents } = this.props;
    
    if (((new TextEncoder().encode(event.target.value))).length <= 280) {
      setNewFardelContents({
        ...newFardelContents,
        contentsText: event.target.value,
      });
    } else {
      setNewFardelContents({
        ...newFardelContents,
        contentsText: newFardelContents.contentsText,
      });
    }
  }

  newFardelIpfsInputHandler(event) {
    const { setNewFardelContents, newFardelContents } = this.props;
    
    if (((new TextEncoder().encode(event.target.value))).length <= 256) {
      setNewFardelContents({
        ...newFardelContents,
        ipfsCid: event.target.value,
      });
    } else {
      setNewFardelContents({
        ...newFardelContents,
        ipfsCid: newFardelContents.ipfsCid,
      });
    }
  }

  newFardelPassphraseInputHandler(event) {
    const { setNewFardelContents, newFardelContents } = this.props;
    
    if (((new TextEncoder().encode(event.target.value))).length <= 256) {
      setNewFardelContents({
        ...newFardelContents,
        passphrase: event.target.value,
      });
    } else {
      setNewFardelContents({
        ...newFardelContents,
        passphrase: newFardelContents.passphrase,
      });
    }
  }

  newFardelContentsPriceInputHandler(event) {
    const { setNewFardelContents, newFardelContents } = this.props;

    if (event.target.value === '') {
      setNewFardelContents({
        ...newFardelContents,
        cost: '',
      });
    } else {
      const regex = /^[0-5]([\.][0-9]{0,6})?/;
      const floatValue = parseFloat(event.target.value);
      if (regex.test(event.target.value) && floatValue >=0 && floatValue <= 5) {
        setNewFardelContents({
          ...newFardelContents,
          cost: event.target.value,
        });
      }
    }
  }

  thumbnailChangedHandler(event) {
    const { setNewFardelThumbnail } = this.props;

    let fileInput = false
    if (event.target.files[0]) {
      fileInput = true
    }
    if (fileInput) {
      try {
        Resizer.imageFileResizer(
          event.target.files[0],
          60,
          60,
          'JPEG',
          100,
          0,
          uri => {
            setNewFardelThumbnail(uri)
          },
          'base64',
          null,
          null,
        );
      } catch(err) {
        console.log(err)
      }
    }
  }

  handleCarryButtonClick() {
    const { 
      clearNewFardel, 
      executeCarryFardel, 
      newFardelMessage, 
      newFardelContents 
    } = this.props;
    
    executeCarryFardel(
      newFardelMessage, 
      newFardelContents.contentsText, 
      newFardelContents.ipfsCid, 
      newFardelContents.passphrase,
      newFardelContents.cost
    );
    clearNewFardel();
  }

  render() {
    const { classes, keplrEnabled, newFardelThumbnail, newFardelMessage, newFardelContents } = this.props;

    let inner_text;
    if (keplrEnabled) {
      inner_text = "keplr is enabled";
    } else {
      inner_text = "keplr is not enabled";
    }

    return (
      <div className={classes.root}>
        <Grid container spacing={1} alignItems={"center"}>
          <Grid item xs={8}>
            <TextField
              id="fardel-message"
              label="Message"
              style={{ margin: 8 }}
              placeholder="What do you want to share?"
              value={newFardelMessage}
              fullWidth
              multiline
              rowsMax={4}
              margin="normal"
              InputLabelProps={{
                shrink: true,
              }}
              variant="filled"
              onChange={(e) => this.newFardelMessageChangedHandler(e) }
            />
          </Grid>
          <Grid item xs={2}>
            <Box className={classes.buttonBox} display="flex" justifyContent="flex-end" p={1}>
              <Button
                variant="contained"
                color="default"
                className={classes.button}
                startIcon={<BackupTwoToneIcon />}
                onClick={() => this.handleCarryButtonClick()}
              >
                Carry
              </Button>
            </Box>
          </Grid>
          <Grid item xs={2}>
          
          </Grid>
          <Grid item xs={8}>
            <Paper className={classes.paper} variant="outlined">

              <Typography variant="h6">
                Fardel contents
              </Typography>
              <TextField
                id="fardel-contents"
                className={classes.textFieldContents}
                label="Contents"
                value={newFardelContents.contentsText}
                placeholder="What's inside?"
                fullWidth
                multiline
                rowsMax={4}
                margin="normal"
                InputLabelProps={{
                  shrink: true,
                }}
                onInput={(e) => this.newFardelContentsTextInputHandler(e) }
                error={(newFardelContents.ipfsCid === '') && (newFardelContents.contentsText === '')}
              />
              <TextField 
                id="fardel-ipfs" 
                className={classes.textFieldContents}
                label="IPFS CID"
                fullWidth
                margin="normal"
                value={newFardelContents.ipfsCid}
                InputProps={{
                  startAdornment: <InputAdornment position="start">{"CID"}</InputAdornment>,
                }}
                onInput={(e) => this.newFardelIpfsInputHandler(e) }
                error={(newFardelContents.ipfsCid === '') && (newFardelContents.contentsText === '')}
              />
              <TextField
                id="fardel-passphrase"
                className={classes.textFieldContents}
                label="Passphrase"
                value={newFardelContents.passphrase}
                placeholder="decrypts file (e.g., GPG passphrase)"
                fullWidth
                multiline
                rowsMax={1}
                margin="normal"
                InputLabelProps={{
                  shrink: true,
                }}
                onInput={(e) => this.newFardelPassphraseInputHandler(e) }
              />
              <TextField 
                id="fardel-price" 
                className={classes.textFieldPrice}
                label="Price to unpack"
                helperText="Limit 5 scrt"
                type="number"
                value={newFardelContents.cost}
                InputProps={{
                  startAdornment: <InputAdornment position="start">scrt</InputAdornment>,
                }}
                onInput={(e) => this.newFardelContentsPriceInputHandler(e) }
                error={newFardelContents.cost === ''}
              />
            </Paper>
          </Grid>
          <Grid item xs={2} />
          <Grid item xs={12}> 
            <Divider className={classes.divider} flexItem/>
          </Grid>
          <Grid item xs={12}>
            <div>{inner_text}</div>
          </Grid>
        </Grid>
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
    newFardelMessage: state.newFardelMessage,
    newFardelContents: state.newFardelContents,
    newFardelThumbnail: state.newFardelThumbnail,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
    executeCarryFardel: (publicMessage, contentsText, ipfsCid, passphrase, cost) =>
        { dispatch(executeCarryFardel(publicMessage, contentsText, ipfsCid, passphrase, cost)) },
    setNewFardelMessage: (value) => { dispatch(setNewFardelMessage(value)) },
    setNewFardelContents: (value) => { dispatch(setNewFardelContents(value)) },
    setNewFardelThumbnail: (thumbnail) => { dispatch(setNewFardelThumbnail(thumbnail)) },
    clearNewFardel: () => { dispatch(clearNewFardel()) },
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(NewFardel));
